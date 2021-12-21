use crate::scenes::scene_stack::SceneTransition;

// use coffee::load::{Task};

use specs::{World};

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use anyhow::Result;

use serde::Deserialize;
use serde_json::Value;
use crate::input::Input;
use crate::loading::{DrawTask, GenTask};
use luminance_glfw::GL33Context;

pub mod scene_stack;

pub const SCENES_DIR: &str = "scenes/";
pub const SCENE_LOADER_FILE_ID: &str = "scene_loader";

#[derive(Deserialize, Debug)]
pub struct SceneLoaderJSON {
    pub entity_paths: Vec<String>,
    pub scene_values: Value
}

pub trait Scene<T: Input + Debug>: Debug {
    // Instance Methods
    fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<SceneTransition<T>>;
    fn draw(&mut self, ecs: Arc<RwLock<World>>) -> Result<()>;
    fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &T) -> Result<()>;
    fn get_name(&self) -> String;
    fn is_finished(&self, ecs: Arc<RwLock<World>>) -> Result<bool>;
}

pub trait SceneLoader<T: Input + Debug>: Debug {
    // TODO: consider changing this to consume self so we don't have to worry about lifetimes in load functions.
    fn load_scene(&self) -> GenTask<Box<dyn Scene<T>>>;
}