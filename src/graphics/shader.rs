use luminance_front::shader::Program;
use crate::loading::{GenTask, DrawTask};
use crate::load::{load_deserializable_from_file, LoadError};
use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, debug};
use crate::graphics::shader::ShaderLoadError::{DeserializeError, ContextWriteError, FileReadError};
use luminance::context::GraphicsContext;
use crate::graphics::render::sprite_renderer::SpriteRenderError::ShaderProgramBuildError;
use std::fs::read_to_string;
use crate::graphics::render::sprite_renderer::DefaultShaderUniform;
use serde::Deserialize;

pub const SHADER_LOAD_ID: &str = "shader";

const VS: &'static str = include_str!("../texture-vs.glsl");
const FS: &'static str = include_str!("../texture-fs.glsl");

pub struct ShaderLoader {
    path: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShaderJSON {
    vertex: String,
    tess: String,
    geometry: String,
    fragment: String,
}

impl ShaderLoader {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new(file_path: String) -> Self {
        Self {
            path: file_path
        }
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load<Sem, Out, Uni>(&self) -> DrawTask<Program<Sem, Out, Uni>> {
        DrawTask::new(|(ecs, context)| {
            let path = self.path.clone();
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

            let mut context = context.write()
                .map_err(|_| {
                    #[cfg(feature = "trace")]
                    error!("Failed to write acquire lock for context");

                    ContextWriteError
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

            let ts = read_to_string(json.tess.clone())
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to read Tess Shader file: {:?}", json.tess.clone());

                    FileReadError {
                        source: e,
                        path: json.tess.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Read in Tess Shader from file: {:?}", json.tess.clone());

            let gs = read_to_string(json.geometry.clone())
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to read Geometry Shader file: {:?}", json.geometry.clone());

                    FileReadError {
                        source: e,
                        path: json.geometry.clone()
                    }
                })?;
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

            Ok(context.new_shader_program()
                .from_strings(&vs, ts, gs, &fs)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to create Shader Program from Shader files");

                    ShaderProgramBuildError {
                        source: e,
                        vs: json.vertex.clone(),
                        ts: json.tess.clone(),
                        gs: json.geometry.clone(),
                        fs: json.fragment.clone()
                    }
                })?
                .ignore_warnings())
        })
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load_default() -> DrawTask<Program<(), (), DefaultShaderUniform>> {
        DrawTask::new(|(ecs, context)| {
            Ok(context
                .new_shader_program()
                .from_strings(VS, None, None, FS)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to build shader program from shader files. VertexShader: ({:?}), TessShader: ({:?}), GeometryShader: ({:?}), FragmentShader: {:?}", VS.to_string(), String::from(""), String::from(""), FS.to_string());

                    ShaderProgramBuildError {
                        source: e,
                        vs: VS.to_string(),
                        ts: String::from(""),
                        gs: String::from(""),
                        fs: FS.to_string()
                    }
                })?
                .ignore_warnings())
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

    #[error("Failed to read shader program from file: {path}")]
    FileReadError {
        source: std::io::Error,
        path: String
    }
}