use crate::scenes::scene_stack::SceneTransition;

use coffee::graphics::{Window, Frame};
use coffee::{Timer};
use coffee::input::Input;
use coffee::load::{Task};

use specs::{World};

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use anyhow::Result;

use serde::Deserialize;
use serde_json::Value;

pub mod scene_stack;

pub const SCENES_DIR: &str = "scenes/";
pub const SCENE_LOADER_JSON_FILE_ID: &str = "scene_loader_json";

#[derive(Deserialize, Debug)]
pub struct SceneLoaderJSON {
    pub entity_paths: Vec<String>,
    pub scene_values: Value
}

pub trait Scene<T: Input + Debug>: Debug {
    // Instance Methods
    fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<SceneTransition<T>>;
    fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer) -> Result<()>;
    fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut T, window: &mut Window) -> Result<()>;
    fn get_name(&self) -> String;
    fn is_finished(&self) -> Result<bool>;
}

pub trait SceneLoader<T: Input + Debug>: Debug {
    fn load_scene(&self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<Box<dyn Scene<T>>>;
}