use luminance_front::vertex::Semantics;
use luminance_front::shader::{UniformInterface, Program};
use luminance::backend::shader::Shader;
use luminance::tess::{TessVertexData, TessIndex};
use luminance_front::tess::{Interleaved, Deinterleaved, DeinterleavedData, Tess};
use std::marker::PhantomData;
use luminance_front::render_state::RenderState;
use luminance_front::pipeline::Pipeline;
use luminance_front::shading_gate::ShadingGate;
use glam::Mat4;
use specs::World;
use crate::graphics::render::sprite_renderer::SpriteRenderError;
use luminance_front::context::GraphicsContext;
use crate::loading::DrawTask;

pub mod sprite_renderer;
pub(crate) mod deserializations;

pub trait ShaderTypes {
    type Semantics: Semantics;
    type ReturnValue;
    type UniformInterface;
}

pub trait Renderer {
    type S: ShaderTypes;

    fn load(path: String) -> DrawTask<Self> where Self: Sized;

    fn render(
        &mut self,
        pipeline: &Pipeline,
        shd_gate: &mut ShadingGate,
        proj_matrix: &Mat4,
        view: &Mat4,
        world: &World,
    ) -> Result<<<Self as Renderer>::S as ShaderTypes>::ReturnValue, SpriteRenderError>;
}