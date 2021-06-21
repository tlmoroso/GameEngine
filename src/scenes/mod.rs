use crate::scenes::scene_stack::SceneTransition;

// use coffee::load::{Task};

use specs::{World};

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use anyhow::Result;

use serde::Deserialize;
use serde_json::Value;
use crate::input::Input;
use crate::loading::{Task};

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
    fn update(&mut self, ecs: &mut World) -> Result<SceneTransition<T>>;
    fn draw(&mut self, ecs: &mut World) -> Result<()>;
    fn interact(&mut self, ecs: &mut World, input: &T) -> Result<()>;
    fn get_name(&self) -> String;
    fn is_finished(&self) -> Result<bool>;
}

pub trait SceneLoader<T: Input + Debug>: Debug {
    fn load_scene(&self, ecs: &mut World) -> Task<Box<dyn Scene<T>>>;
}