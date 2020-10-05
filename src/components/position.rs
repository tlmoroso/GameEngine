use specs::{Component, World};
use specs::storage::VecStorage;

use coffee::graphics::Window;
use coffee::load::Task;

use crate::load::{Loadable, ComponentLoadable};

use serde::Deserialize;
use serde_json::Value;
use crate::components::ComponentType;
use std::sync::{RwLock, Arc};

pub const POSITION_FILE_ID: &str = "position";

#[derive(Deserialize, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Loadable for Position {}
impl ComponentLoadable for Position {}

impl Position {
    pub fn load(_ecs: Arc<RwLock<World>>, _window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<ComponentType> {
        Task::new(|| {
            let position: Position = serde_json::from_value(json_value)
                .expect("ERROR: could not translate JSON value to Position in Position::load");
            Ok(ComponentType::Position(position))
        })

    }
}
