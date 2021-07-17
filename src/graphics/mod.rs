use anyhow::Result;
use luminance_glfw::GL33Context;

pub mod texture;
pub mod render;
pub mod transform;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Handle(pub String);

// pub(crate) fn draw(context: &mut GL33Context) -> Result<()> {
//     let back_buffer = context.back_buffer()?;
// }