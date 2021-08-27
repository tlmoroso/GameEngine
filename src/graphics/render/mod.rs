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

pub mod sprite_renderer;
pub(crate) mod deserializations;

pub trait ShaderTypes {
    type Semantics: Semantics;
    type ReturnValue;
    type UniformInterface/*: UniformInterface<B>*/;
}

// pub trait TessTypes {
    // type Vertex: TessVertexData<Interleaved, Data=Vec<Self::Vertex>>;
    // type Index: TessIndex;
    // type Instance: TessVertexData<Interleaved, Data=Vec<Self::Instance>>;
// }

// pub struct InterleavedTess<V,I,W>
//     where V: TessVertexData<Interleaved, Data=Vec<V>>,
//           I: TessIndex,
//           W: TessVertexData<Interleaved, Data=Vec<W>> {
//     phantom_vert: PhantomData<V>,
//     phantom_ind: PhantomData<I>,
//     phantom_inst: PhantomData<W>
// }

// impl<V,I,W> TessTypes for InterleavedTess<V,I,W>
//     where V: TessVertexData<Interleaved, Data=Vec<V>>,
//           I: TessIndex,
//           W: TessVertexData<Interleaved, Data=Vec<W>> {
//     // type Storage = Interleaved;
//     // type Data = Vec<V>;
//     type Vertex = V;
//     type Index = I;
//     type Instance = W;
// }

// pub struct DeInterleavedTess<V,I,W>
//     where V: TessVertexData<Deinterleaved, Data=Vec<DeinterleavedData>>,
//           I: TessIndex,
//           W: TessVertexData<Deinterleaved, Data=Vec<DeinterleavedData>> {
//     phantom_vert: PhantomData<V>,
//     phantom_ind: PhantomData<I>,
//     phantom_inst: PhantomData<W>
// }

// impl<V,I,W> TessTypes for DeInterleavedTess<V,I,W>
//     where V: TessVertexData<Deinterleaved, Data=Vec<DeinterleavedData>>,
//           I: TessIndex,
//           W: TessVertexData<Deinterleaved, Data=Vec<DeinterleavedData>> {
//     // type Storage = Deinterleaved;
//     // type Data = Vec<Deinterleaved>;
//     type Vertex = V;
//     type Index = I;
//     type Instance = W;
// }

// pub enum TessType<VI,II,WI,VD,ID,WD> {
//     Interleaved(InterleavedTess<VI,II,WI>),
//     Deinterleaved(DeInterleavedTess<VD,ID,WD>)
// }

pub trait Renderer {
    // type T: TessTypes;
    type S: ShaderTypes;

    fn new(
        shader: Program<
            <<Self as Renderer>::S as ShaderTypes>::Semantics,
            <<Self as Renderer>::S as ShaderTypes>::ReturnValue,
            <<Self as Renderer>::S as ShaderTypes>::UniformInterface
        >,
        tess: Tess<
            // <<Self as Renderer<B>>::T as TessTypes>::Vertex,
            // <<Self as Renderer<B>>::T as TessTypes>::Index,
            // <<Self as Renderer<B>>::T as TessTypes>::Instance,
            (),
            (),
            (),
            Interleaved
        >,
        state: RenderState
    ) -> Self;
    fn render(
        &mut self,
        pipeline: &Pipeline,
        shd_gate: &mut ShadingGate,
        proj_matrix: &Mat4,
        view: &Mat4,
        world: &World,
    ) -> Result<<<Self as Renderer>::S as ShaderTypes>::ReturnValue, SpriteRenderError>;
}