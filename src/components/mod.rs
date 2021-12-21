// pub mod drawables;
// pub mod audibles;

use specs::{World};

use std::fmt::{Debug};

use anyhow::Result;

use crate::load::JSONLoad;
use specs::world::LazyBuilder;
use luminance_glfw::GL33Context;
use std::sync::{Arc, Mutex, RwLock};

pub const COMPONENTS_DIR: &str = "components/";

pub trait ComponentLoader: Debug {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized;
    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: Arc<RwLock<World>>) -> Result<LazyBuilder<'a>>;
    fn set_value(&mut self, new_value: JSONLoad) -> Result<()>;
    fn get_component_name(&self) -> String;
}

pub trait ComponentMux {
    fn map_json_to_loader(json: JSONLoad) -> Result<Box<dyn ComponentLoader>>;
}