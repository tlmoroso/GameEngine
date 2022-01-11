use luminance_front::shader::{Program, UniformInterface, TessellationStages, ProgramError, Stage, StageType};
use crate::loading::{GenTask};
use crate::load::{load_deserializable_from_file, LoadError};
use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, debug};
use crate::graphics::Context;
use crate::graphics::shader::ShaderLoadError::{DeserializeError, ContextWriteError, WorldWriteError, FileReadError, ShaderProgramBuildError};
use luminance::context::GraphicsContext;
use std::fs::read_to_string;
use crate::graphics::render::sprite_renderer::{DefaultSpriteShaderUniform};
use serde::Deserialize;
use luminance_front::vertex::Semantics;
use std::marker::PhantomData;
use luminance::backend::shader::Shader;

pub const SHADER_LOAD_ID: &str = "shader";

const VS: &'static str = include_str!("./texture-vs.glsl");
const FS: &'static str = include_str!("./texture-fs.glsl");

#[derive(Debug, Clone)]
pub struct ShaderLoader {
    path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShaderJSON {
    vertex: String,
    tess_control: Option<String>,
    tess_eval: Option<String>,
    geometry: Option<String>,
    fragment: String,
}

impl ShaderLoader {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new(file_path: String) -> Self {
        Self {
            path: file_path,
            // context: PhantomData,
        }
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load<Sem, Out, Uni>(&self) -> GenTask<Program<Sem, Out, Uni>>
        where Sem: 'static + Semantics,
              Out: 'static,
              Uni: 'static + UniformInterface<luminance_front::Backend> {
        let path = self.path.clone();

        GenTask::new(move |ecs| {
            #[cfg(feature = "trace")]
            debug!("Loading Shader Program from file: {:?}", path.clone());

            let json: ShaderJSON = load_deserializable_from_file(&path, SHADER_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load Shader JSON from file: {:?}", path.clone());
                    DeserializeError {
                        source: e,
                        file_path: path.clone()
                    }
                })?;

            let fs = read_to_string(json.fragment.clone())
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to read Fragment Shader file: {:?}", json.fragment.clone());

                    FileReadError {
                        source: e,
                        path: json.fragment.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Read in Fragment Shader from file: {:?}", json.fragment.clone());

            let ts_c =
                if let Some(path) = &json.tess_control {
                    read_to_string(path)
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to read Tess Control Shader file: {:?}", path.clone());

                            FileReadError {
                                source: e,
                                path: path.clone()
                            }
                        })?
                } else {
                    String::new()
                };
            #[cfg(feature = "trace")]
            debug!("Read in Tess Control Shader from file: {:?}", json.tess_control.clone());

            let ts_e =
                if let Some(path) = &json.tess_eval {
                    read_to_string(path)
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to read Tess Eval Shader file: {:?}", path.clone());

                            FileReadError {
                                source: e,
                                path: path.clone()
                            }
                        })?
                } else {
                    String::new()
                };
            #[cfg(feature = "trace")]
            debug!("Read in Tess Evaluation Shader from file: {:?}", json.tess_eval.clone());

            let tess_stages =
                if json.tess_control.is_some() && json.tess_eval.is_some() {
                    Some(TessellationStages {
                        control: ts_c.as_str(),
                        evaluation: ts_e.as_str()
                    })
                } else {
                    None
                };

            let geometry_shader =
                if let Some(path) = &json.geometry {
                    read_to_string(path)
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to read Geometry Shader file: {:?}", json.geometry);

                            FileReadError {
                                source: e,
                                path: path.clone()
                            }
                        })?
                } else {
                    String::new()
                };
            let gs =
                if json.geometry.is_some() {
                    Some(geometry_shader.as_str())
                } else {
                    None
                };

            #[cfg(feature = "trace")]
            debug!("Read in Geometry Shader from file: {:?}", json.geometry.clone());

            let vs = read_to_string(json.vertex.clone())
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to read Vertex Shader file: {:?}", json.vertex.clone());

                    FileReadError {
                        source: e,
                        path: json.vertex.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Read in Vertex Shader from file: {:?}", json.vertex.clone());

            let ecs = ecs.write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for World");

                    WorldWriteError
                })?;

            let context = ecs.fetch::<Context>();

            let mut context = context.0
                .write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for Context");

                    ContextWriteError
                })?;

            let built_program = context.new_shader_program()
                .from_strings(&vs, tess_stages, gs, &fs)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to create Shader Program from Shader files");

                    ShaderProgramBuildError {
                        source: e,
                        vs: json.vertex.clone(),
                        ts_c: json.tess_control.clone(),
                        ts_e: json.tess_eval.clone(),
                        gs: json.geometry.clone(),
                        fs: json.fragment.clone()
                    }
                })?;

            #[cfg(feature = "trace")]
            debug!("Shader program built. Ignoring Warning from shader program: {:?}", built_program.warnings);

            Ok(built_program.ignore_warnings())
        })
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load_default() -> GenTask<Program<(), (), DefaultSpriteShaderUniform>> {
        GenTask::new(|ecs| {
            let ecs = ecs.write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for World");

                    WorldWriteError
                })?;

            let context = ecs.fetch::<Context>();

            let mut context = context.0
                .write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for Context");

                    ContextWriteError
                })?;

            let built_program = context
                .new_shader_program()
                .from_strings(VS, None, None, FS)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to build shader program from shader files. VertexShader: ({:?}), TessShader: ({:?}), GeometryShader: ({:?}), FragmentShader: {:?}", VS.to_string(), String::from(""), String::from(""), FS.to_string());

                    ShaderProgramBuildError {
                        source: e,
                        vs: VS.to_string(),
                        ts_c: None,
                        ts_e: None,
                        gs: None,
                        fs: FS.to_string()
                    }
                })?;

               #[cfg(feature = "trace")]
               debug!("Shader program built. Ignoring Warnings from shader program: {:?}", built_program.warnings);

                Ok(built_program.ignore_warnings())
        })
    }
}

#[derive(Error, Debug)]
pub enum ShaderLoadError {
    #[error("Failed to load deserializable from file: {file_path}")]
    DeserializeError {
        source: LoadError,
        file_path: String
    },

    #[error("Failed to get write lock for context")]
    ContextWriteError,
    #[error("Failed to get write lock for World")]
    WorldWriteError,

    #[error("Failed to read shader program from file: {path}")]
    FileReadError {
        source: std::io::Error,
        path: String
    },

    #[error("Failed to build new shader program from shaders.\n \tVertexShader:\n \t({vs:?}),\n \tTessControlShader: ({ts_c:?}),\n \tTessEvalShader: ({ts_e:?}),\n \tGeometryShader: ({gs:?}),\n \tFragmentShader: {fs:?} ")]
    ShaderProgramBuildError {
        source: ProgramError,
        vs: String,
        ts_c: Option<String>,
        ts_e: Option<String>,
        gs: Option<String>,
        fs: String
    }
}