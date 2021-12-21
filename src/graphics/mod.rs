use anyhow::Result;
use luminance_glfw::GL33Context;
use std::sync::{Arc, RwLock};

pub mod texture;
pub mod render;
pub mod transform;
pub mod shader;
pub mod tess;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Handle(pub String);

pub struct Context(pub Arc<RwLock<GL33Context>>);

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

impl Context {
    pub fn new(context: GL33Context) -> Self {
        Self { 0: Arc::new(RwLock::new(context)) }
    }
}

// pub(crate) fn draw(context: &mut GL33Context) -> Result<()> {
//     let back_buffer = context.back_buffer()?;
// }