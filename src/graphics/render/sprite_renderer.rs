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
use crate::graphics::render::sprite_renderer::SpriteRenderError::{FailedToBind, TessRenderError, RenderGateError};

use thiserror::Error;
use luminance_front::tess::{Interleaved, TessError};
use std::ops::DerefMut;
use luminance_glfw::GL33Context;
use glam::Mat4;
use std::error::Error;
use luminance_front::depth_test::DepthComparison::Always;

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

// pub fn new_shader<B>(context: &mut B) -> Program<(), (), ShaderUniform>
//     where B: GraphicsContext, <B as GraphicsContext>::Backend:Shader
// {
//     context
//         .new_shader_program::<(), (), ShaderUniform>()
//         .from_strings(VS, None, None, FS)
//         .expect("Program creation")
//         .ignore_warnings()
// }

pub struct SpriteRenderer
{
    pub render_state: RenderState,
    pub tess: Tess<()>,
    pub shader: Program<(), (), ShaderUniform>,
}

impl SpriteRenderer {
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
        let tess = context
            .new_tess()
            .set_render_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()
            .expect("Tess creation");

        let shader = context
            .new_shader_program()
            .from_strings(VS, None, None, FS)
            .expect("Couldn't create new shader")
            .ignore_warnings();

        SpriteRenderer {
            render_state,
            tess,
            shader,
        }
    }

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
            iface.set(&uni.projection, proj_matrix.to_cols_array_2d());
            iface.set(&uni.view, view.to_cols_array_2d());

            let (mut textures, transforms, mut texture_dict): (WriteStorage<Texture>, ReadStorage<Transform>, Write<TextureDict>) = world.system_data();

            for (tex_handle, transform) in (&mut textures, &transforms).join() {
                if let Some(texture) = texture_dict.get_mut(tex_handle) {
                    let bound_tex = pipeline.bind_texture(texture)
                        .map_err(|e| {
                            FailedToBind {
                                texture: tex_handle.clone(),
                                source: e
                            }
                        })?;

                    iface.set(&uni.tex, bound_tex.binding());
                    let model = transform.to_model();
                    iface.set(&uni.model, model.to_cols_array_2d());

                    rdr_gate.render(render_state, |mut tess_gate| {
                        tess_gate.render(tess)
                            .map_err(|e| {
                                TessRenderError {
                                    source: e
                                }
                            })
                    })
                        .map_err(|e| {
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
    }
}

impl From<PipelineError> for SpriteRenderError {
    fn from(e: PipelineError) -> Self {
        Self::PipelineRenderError { source: e }
    }
}