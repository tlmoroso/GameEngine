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

use crate::graphics::texture::Texture;
use crate::graphics::transform::Transform;
use crate::globals::texture_dict::TextureDict;
use crate::graphics::render::sprite_renderer::SpriteRenderError::{FailedToBind, TessRenderError, RenderGateError, TessBuildError, ShaderProgramBuildError};

use thiserror::Error;
use luminance_front::tess::{Interleaved, TessError};
use std::ops::DerefMut;
use luminance_glfw::GL33Context;
use glam::Mat4;
use std::error::Error;
use luminance_front::depth_test::DepthComparison::Always;

#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};
use luminance_front::shader::ProgramError;

const VS: &'static str = include_str!("../texture-vs.glsl");
const FS: &'static str = include_str!("../texture-fs.glsl");

#[derive(Debug, UniformInterface)]
pub struct ShaderUniform {
    /// PROJECTION matrix in MVP
    projection: Uniform<[[f32; 4]; 4]>,
    /// VIEW matrix in MVP
    view: Uniform<[[f32; 4]; 4]>,
    /// MODEL matrix in MVP
    model: Uniform<[[f32; 4]; 4]>,

    /// Texture for the texture.
    tex: Uniform<TextureBinding<Dim2, Unsigned>>,
}

pub struct SpriteRenderer
{
    pub render_state: RenderState,
    pub tess: Tess<()>,
    pub shader: Program<(), (), ShaderUniform>,
}

impl SpriteRenderer {
    #[cfg_attr(feature = "trace", instrument(skip(context)))]
    pub fn new(context: &mut GL33Context) -> SpriteRenderer {
        let render_state = RenderState::default()
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
            );
        #[cfg(feature = "trace")]
        debug!("Created render state for renderer: {:?}", render_state.clone());

        let tess = context
            .new_tess()
            .set_render_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()
            .map_err(|e| {
                TessBuildError {
                    source: e
                }
            })?;

        #[cfg(feature = "trace")]
        debug!("Created tesselation for drawing sprites");

        let shader = context
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
            .ignore_warnings();



        SpriteRenderer {
            render_state,
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
        let shader = &mut self.shader;
        let render_state = &self.render_state;
        let tess = &self.tess;

        shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
            #[cfg(feature = "trace")]
            debug!("Entering shading gate.");

            iface.set(&uni.projection, proj_matrix.to_cols_array_2d());
            iface.set(&uni.view, view.to_cols_array_2d());
            #[cfg(feature = "trace")]
            debug!("Setting uniform values for projection and view matrices using ProgramInterface");

            let (mut textures, transforms, mut texture_dict): (WriteStorage<Texture>, ReadStorage<Transform>, Write<TextureDict>) = world.system_data();
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

                    rdr_gate.render(render_state, |mut tess_gate| {
                        #[cfg(feature = "trace")]
                        debug!("Entering render gate.");

                        tess_gate.render(tess)
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
        texture: Texture,
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

    #[error("Failed to build Tesselation")]
    TessBuildError {
        source: TessError
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