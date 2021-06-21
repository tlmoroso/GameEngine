pub mod mouse;
pub mod keyboard;
pub mod multi_input;

use glfw::{WindowEvent};

pub trait Input: Send + Sync {
    fn new() -> Self;

    fn update(&mut self, event: WindowEvent);

    fn clear(&mut self);
}
