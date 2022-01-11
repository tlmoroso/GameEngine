#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};

use luminance_glfw::GL33Context;
use crate::graphics::render::sprite_renderer::SpriteRenderError;
use luminance_front::tess::{Tess, Mode, TessError, Interleaved};
use luminance::context::GraphicsContext;
use thiserror::Error;
use crate::graphics::tess::TessLoadError::{TessBuildError, DeserializeError, ContextWriteError, WorldWriteLockError};
use serde::Deserialize;
use crate::loading::GenTask;
use crate::load::{load_deserializable_from_file, LoadError};
use anyhow::{Error};
use luminance::tess::TessVertexData;
use std::fmt::Debug;
use crate::graphics::Context;

pub const TESS_LOAD_ID: &str = "tess";

#[derive(Debug, Clone)]
pub struct TessLoader {
    file_path: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct TessJSON {
    #[serde(default)]
    mode: Option<ModeDef>,
    #[serde(default)]
    render_vertices_len: Option<usize>,
    #[serde(default)]
    render_instances_len: Option<usize>,
    #[serde(default)]
    primitive_restart_index: Option<u32>,
    #[serde(default)]
    attributes: Option<Vec<u32>>,
    #[serde(default)]
    instance_attributes: Option<Vec<u32>>,
}

impl TessLoader {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new(file_path: String) -> TessLoader {
        Self {
            file_path
        }
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load(&self) -> GenTask<Tess<(),(),(),Interleaved>> {
        let path = self.file_path.clone();

        GenTask::new(move |ecs| {
            #[cfg(feature = "trace")]
            debug!("Loading Tess from file: {:?}", path.clone());

            let json: TessJSON = load_deserializable_from_file(&path, TESS_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load deserializable from file: {:?}", path.clone());

                    DeserializeError {
                        source: e,
                        file_path: path.clone()
                    }
                })?;

            #[cfg(feature = "trace")]
            debug!("Loaded json from file: {:?}", json.clone());

            let ecs = ecs.write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for World");

                    WorldWriteLockError
                })?;

            let context = ecs.fetch::<Context>();

            let mut context = context.0
                .write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for Context");

                    ContextWriteError
                })?;

            let mut tess_builder = context.new_tess();
            #[cfg(feature = "trace")]
            debug!("Created Tess builder");

            if let Some(mode) = json.mode {
                #[cfg(feature = "trace")]
                debug!("Setting Tess mode: {:?}", mode.clone());

                tess_builder = tess_builder.set_mode(Mode::from(mode))
            }

            if let Some(render_vertex_nb) = json.render_vertices_len {
                #[cfg(feature = "trace")]
                debug!("Setting default number of vertices to render: {:?}", render_vertex_nb);

                tess_builder = tess_builder.set_render_vertex_nb(render_vertex_nb)
            }
            if let Some(render_instance_nb) = json.render_instances_len {
                #[cfg(feature = "trace")]
                debug!("Setting default number of instances to render: {:?}", render_instance_nb);

                tess_builder = tess_builder.set_render_instance_nb(render_instance_nb)
            }

            tess_builder.build()
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to build Tess");

                    Error::new(TessBuildError {
                        source: e
                    })
                })
        })
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load_default() -> GenTask<Tess<(),(),(),Interleaved>> {
        GenTask::new(|ecs| {
            #[cfg(feature = "trace")]
            debug!("Loading Default Tess");

            let ecs = ecs.write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for World");

                    WorldWriteLockError
                })?;

            let context = ecs.fetch::<Context>();

            let mut context = context.0
                .write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for Context");

                    ContextWriteError
                })?;

            Ok(context
                .new_tess()
                .set_render_vertex_nb(4)
                .set_mode(Mode::TriangleFan)
                .build()
                .map_err(|e| {
                    TessBuildError {
                        source: e
                    }
                })?)
        })
    }
}

#[derive(Error, Debug)]
pub enum TessLoadError {
    #[error("Failed to build Tesselation")]
    TessBuildError {
        source: TessError
    },

    #[error("Failed to load TessJSON from file: {file_path}")]
    DeserializeError {
        source: LoadError,
        file_path: String
    },

    #[error("Failed to acquire write lock for Context")]
    ContextWriteError,
    #[error("Failed to acquire write lock for Context")]
    WorldWriteLockError,
}

#[derive(Deserialize,Debug,Clone)]
enum ModeDef {
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleFan,
    TriangleStrip,
    Patch(usize)
}

impl From<ModeDef> for Mode {
    fn from(m: ModeDef) -> Self {
        match m {
            ModeDef::Point => Mode::Point,
            ModeDef::Line => Mode::Line,
            ModeDef::LineStrip => Mode::LineStrip,
            ModeDef::Triangle => Mode::Triangle,
            ModeDef::TriangleFan => Mode::TriangleFan,
            ModeDef::TriangleStrip => Mode::TriangleStrip,
            ModeDef::Patch(p) => Mode::Patch(p)
        }
    }
}