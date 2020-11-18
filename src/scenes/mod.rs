use crate::scenes::scene_stack::SceneTransition;

use coffee::graphics::{Window, Frame};
use coffee::{Timer};
use coffee::load::{Task};

use specs::{World};
use coffee::input::Input;

pub mod scene_stack;

pub const SCENE_STACK_FILE_ID: &str = "scene_stack";
pub const SCENES_DIR: &str = "scenes/";

pub trait Scene<T: Input> {
    // Instance Methods
    fn update(&mut self, ecs: &mut World) -> SceneTransition<T>;
    fn draw(&mut self, ecs: &mut World, frame: &mut Frame, timer: &Timer);
    fn interact(&mut self, ecs: &mut World, input: &mut T, window: &mut Window);
}

pub trait SceneLoader<T: Input> {
    fn load_scene(&self, ecs: &mut World, window: &Window) -> Task<Box<dyn Scene<T>>>;
}