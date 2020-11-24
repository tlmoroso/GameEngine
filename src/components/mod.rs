use specs::{World, EntityBuilder};

use coffee::graphics::Window;

use serde_json::Value;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use crate::load::JSONLoad;
use std::fmt::{Debug};

pub const COMPONENTS_DIR: &str = "components/";

pub trait ComponentLoader: Debug {
    fn load_component(&self, entity_task: EntityBuilder, ecs: Arc<RwLock<World>>, window: &Window) -> Result<EntityBuilder>;
    fn set_value(&mut self, new_value: Value) -> Result<()>;
}

pub trait ComponentMux {
    fn map_json_to_loader(json: JSONLoad) -> Result<Box<dyn ComponentLoader>>;
}