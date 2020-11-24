use crate::scenes::scene_stack::SceneTransition;

use coffee::graphics::{Window, Frame};
use coffee::{Timer};
use coffee::input::Input;
use coffee::load::{Task};

use specs::{World};

use std::sync::{Arc, RwLock};

use anyhow::Result;

pub mod scene_stack;

pub const SCENES_DIR: &str = "scenes/";

pub trait Scene<T: Input> {
    // Instance Methods
    fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<SceneTransition<T>>;
    fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer) -> Result<()>;
    fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut T, window: &mut Window) -> Result<()>;
}

pub trait SceneLoader<T: Input> {
    fn load_scene(&self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<Box<dyn Scene<T>>>;
}