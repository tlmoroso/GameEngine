// use luminance::context::GraphicsContext;
use luminance_front::{
    render_state::RenderState,
    tess::{Tess, Mode},
    shader::{Uniform, Program},
    pipeline::{
        TextureBinding,
        Pipeline,
        PipelineError
    },
    texture::Dim2,
    blending::{
        Blending,
        Equation,
        Factor
    },
    shading_gate::ShadingGate,
    pixel::{Pixel, Unsigned},
    context::GraphicsContext,

};
use luminance_derive::UniformInterface;
use luminance::backend::shader::Shader;

use serde::Deserialize;

use specs::{World, Write, Join, WriteStorage, ReadStorage};

use crate::graphics::texture::TextureHandle;
use crate::graphics::transform::Transform;
use crate::globals::texture_dict::TextureDict;
use crate::graphics::render::sprite_renderer::SpriteRenderError::{FailedToBind, TessRenderError, RenderGateError, ShaderProgramBuildError};

use thiserror::Error;
use luminance_front::tess::{Interleaved, TessError};
use std::ops::DerefMut;
use luminance_glfw::GL33Context;
use glam::Mat4;
use std::error::Error;
use luminance_front::shader::{ProgramError, UniformInterface};
use luminance_front::depth_test::DepthComparison::Always;

#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};

use crate::loading::DrawTask;
use luminance_front::vertex::Semantics;
use luminance::tess::TessVertexData;
use crate::load::{load_deserializable_from_file, LoadError};
use crate::graphics::tess::TessLoader;
use luminance::blending::BlendingMode;
use luminance::depth_test::{DepthComparison, DepthWrite};
use luminance::face_culling::FaceCulling;
use luminance::scissor::ScissorRegion;
use crate::graphics::render::sprite_renderer::SpriteRendererLoadError::{DeserializeError, TessLoadError, ShaderLoadError};
use crate::graphics::shader::ShaderLoader;

pub const RENDER_STATE_LOAD_ID: &str = "render_state";

#[derive(Deserialize)]
#[serde(remote = "RenderState")]
struct RenderStateDef {
    /// Blending configuration.
    blending: Option<BlendingMode>,
    /// Depth test configuration.
    depth_test: Option<DepthComparison>,
    /// Depth write configuration.
    depth_write: DepthWrite,
    /// Face culling configuration.
    face_culling: Option<FaceCulling>,
    /// Scissor region configuration.
    scissor: Option<ScissorRegion>,
}

#[cfg_attr(feature = "trace", instrument)]
pub fn default_sprite_render_state() -> RenderState {
    RenderState::default()
        .set_depth_test(Some(Always))
        .set_blending_separate(
            Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::SrcAlphaComplement,
            },
            Blending {
                equation: Equation::Additive,
                src: Factor::One,
                dst: Factor::Zero,
            },
        )
}

#[derive(Debug, UniformInterface)]
pub struct DefaultSpriteShaderUniform {
    /// PROJECTION matrix in MVP
    projection: Uniform<[[f32; 4]; 4]>,
    /// VIEW matrix in MVP
    view: Uniform<[[f32; 4]; 4]>,
    /// MODEL matrix in MVP
    model: Uniform<[[f32; 4]; 4]>,

    /// Texture for the texture.
    tex: Uniform<TextureBinding<Dim2, Unsigned>>,
}

pub const SPRITE_RENDERER_LOAD_ID: &str = "sprite_renderer";

pub struct SpriteRendererLoader {
    path: String
}

#[derive(Deserialize)]
pub struct SpriteRendererJSON {
    render_state_path: String,
    tess_path: String,
    shader_path: String
}

impl SpriteRendererLoader {
    pub fn new(path: String) -> Self {
        Self {
            path
        }
    }

    pub fn load<T: TessTypes, S: ShaderTypes>(&self) -> DrawTask<SpriteRenderer<T,S>> {
        DrawTask::new(|(ecs, context)| {
            #[cfg(feature = "trace")]
            debug!("Loading Sprite Renderer from file: {:?}", self.path.clone());
            let path = self.path.clone();

            let json: SpriteRendererJSON = load_deserializable_from_file(&path, SPRITE_RENDERER_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load deserializable from file: {:?}", path.clone());

                    DeserializeError {
                        source: e,
                        path: path.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Loaded json from file: {:?}", json.clone());

            let render_state: RenderState = load_deserializable_from_file(&json.render_state_path, RENDER_STATE_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to deserialize Render State from file: {:?}", json.render_state_path.clone());

                    DeserializeError {
                        source: e,
                        path: json.render_state_path.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Loaded Render State: ({:?}) from file: {:?}", render_state.clone(), json.render_state_path.clone());

            let tess = TessLoader::new(json.tess_path)
                .load()
                .execute((ecs.clone(), context.clone()))
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load Tess from file: {:?}", json.tess_path.clone());

                    TessLoadError {
                        source: e,
                        path: json.tess_path.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Loaded Tess from file: {:?}", json.tess_path.clone());

            let shader = ShaderLoader::new(json.shader_path)
                .load()
                .execute((ecs, context))
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load shader from file: {:?}", json.shader_path);

                    ShaderLoadError {
                        source: e,
                        path: json.shader_path.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Loaded shader from file: {:?}", json.shader_path.clone());
            
            Ok(SpriteRenderer {
                render_state,
                tess,
                shader
            })
        })
    }
}

#[derive(Error, Debug)]
pub enum SpriteRendererLoadError {
    #[error("Failed to deserialize file: {path:?}")]
    DeserializeError {
        source: LoadError,
        path: String
    },

    #[error("Failed to acquire write lock for context")]
    ContextWriteError,

    #[error("Failed to load Tess from file: {path}")]
    TessLoadError {
        source: anyhow::Error,
        path: String
    },

    #[error("Failed to load Shader from file: {path}")]
    ShaderLoadError {
        source: anyhow::Error,
        path: String
    }
}

pub trait ShaderTypes {
    type Semantics: Semantics;
    type UniformInterface: UniformInterface<Self::Shader>;
    type Shader: Shader;
}

pub trait TessTypes {
    type Vertex: TessVertexData<Self::Storage>;
    type Instance: TessVertexData<Self::Storage>;
    type Storage: ?Sized;
}

pub struct SpriteRenderer<T,S>
    where T: TessTypes,
          S: ShaderTypes
{
    pub render_state: RenderState,
    pub tess: Tess<T::Vertex,(),T::Instance,T::Storage>,
    pub shader: Program<S::Semantics, Out, S::UniformInterface>,
}

impl<T: TessTypes,S:ShaderTypes> SpriteRenderer<T,S> {
    #[cfg_attr(feature = "trace", instrument(skip(shader, tess)))]
    pub fn new(shader: Program<Sem, Out, Uni>, tess: Tess<V>, state: RenderState) -> SpriteRenderer<T,S> {
        SpriteRenderer {
            render_state: state,
            tess,
            shader,
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, pipeline, shd_gate, world)))]
    pub fn render(
        &mut self,
        pipeline: &Pipeline,
        shd_gate: &mut ShadingGate,
        proj_matrix: &Mat4,
        view: &Mat4,
        world: &World,
    ) -> Result<(), SpriteRenderError> {
        shd_gate.shade(&mut self.shader, |mut iface, uni, mut rdr_gate| {
            #[cfg(feature = "trace")]
            debug!("Entering shading gate.");

            iface.set(&uni.projection, proj_matrix.to_cols_array_2d());
            iface.set(&uni.view, view.to_cols_array_2d());
            #[cfg(feature = "trace")]
            debug!("Setting uniform values for projection and view matrices using ProgramInterface");

            let (mut textures, transforms, mut texture_dict): (WriteStorage<TextureHandle>, ReadStorage<Transform>, Write<TextureDict>) = world.system_data();
            #[cfg(feature = "trace")]
            debug!("Getting all entities with a texture and transform component to draw. Also fetching TextureDict.");

            for (tex_handle, transform) in (&mut textures, &transforms).join() {
                #[cfg(feature = "trace")]
                debug!("Rendering texture: ({:?}) with transform: {:?}", tex_handle.clone(), transform);

                if let Some(texture) = texture_dict.get_mut(tex_handle) {
                    #[cfg(feature = "trace")]
                    debug!("Found texture in dict for given texture handle.");

                    let bound_tex = pipeline.bind_texture(texture)
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to bind texture to pipeline.");

                            FailedToBind {
                                texture: tex_handle.clone(),
                                source: e
                            }
                        })?;

                    iface.set(&uni.tex, bound_tex.binding());
                    let model = transform.to_model();
                    iface.set(&uni.model, model.to_cols_array_2d());
                    #[cfg(feature = "trace")]
                    debug!("Successfully bound texture. Setting texture and model matrix for uniform.");

                    rdr_gate.render(&self.render_state, |mut tess_gate| {
                        #[cfg(feature = "trace")]
                        debug!("Entering render gate.");

                        tess_gate.render(&self.tess)
                            .map_err(|e| {
                                #[cfg(feature = "trace")]
                                error!("Failed to call render on tess gate.");

                                TessRenderError {
                                    source: e
                                }
                            })?;

                        #[cfg(feature = "trace")]
                        debug!("Successfully called render on tess gate.");

                        Ok(())
                    })
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to call render on render gate.");

                            RenderGateError {
                                source: Box::new(e)
                            }
                        })?;
                }
            }

            Ok(())
        })
    }
}

#[derive(Error, Debug)]
pub enum SpriteRenderError {
    #[error("Failed to bind texture={texture:?} to pipeline")]
    FailedToBind {
        texture: TextureHandle,
        source: PipelineError
    },
    
    #[error("An error occurred in the pipeline")]
    PipelineRenderError {
        source: PipelineError    
    },

    #[error("An error occurred while rendering the tess gate")]
    TessRenderError {
        source: TessError
    },

    #[error("An error occurred while rendering the render gate")]
    RenderGateError {
        source: Box<SpriteRenderError>
    },

    #[error("Failed to build new shader program from shaders. VertexShader: ({vs}), TessShader: ({ts}), GeometryShader: ({gs}), FragmentShader: {fs} ")]
    ShaderProgramBuildError {
        source: ProgramError,
        vs: String,
        ts: String,
        gs: String,
        fs: String
    }
}

impl From<PipelineError> for SpriteRenderError {
    fn from(e: PipelineError) -> Self {
        Self::PipelineRenderError { source: e }
    }
}