use specs::{World, EntityBuilder};

use coffee::graphics::Window;
use coffee::load::Task;

use serde_json::Value;

pub const COMPONENTS_DIR: &str = "components/";

pub trait ComponentLoader {
    fn load_component(&self, json_value: Value, entity_task: Task<EntityBuilder>, ecs: &mut World, window: &Window) -> Task<EntityBuilder>;
}

pub trait ComponentMux {
    fn map_id_to_loader(component_id: String) -> Box<dyn ComponentLoader>;
}