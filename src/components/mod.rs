use specs::{World, EntityBuilder, Component};

use coffee::graphics::Window;

use std::sync::{Arc, RwLock};
use std::fmt::{Debug};

use anyhow::Result;

use crate::load::JSONLoad;
use specs::world::LazyBuilder;

pub const COMPONENTS_DIR: &str = "components/";

pub trait ComponentLoader: Debug {
    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>>;
    fn set_value(&mut self, new_value: JSONLoad) -> Result<()>;
    fn get_component_name(&self) -> String;
}

pub trait ComponentMux {
    fn map_json_to_loader(json: JSONLoad) -> Result<Box<dyn ComponentLoader>>;
}