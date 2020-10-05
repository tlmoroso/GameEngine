use specs::{Component, World};
use specs::storage::HashMapStorage;
use coffee::graphics::Window;
use coffee::load::Task;
use crate::load::{Loadable, ComponentLoadable};
use serde_json::Value;
use crate::components::ComponentType;
use std::sync::{RwLock, Arc};

pub const PLAYER_CONTROL_FILE_ID: &str = "player_control";

#[derive(Debug)]
pub struct PlayerControl {}

impl Component for PlayerControl {
    type Storage = HashMapStorage<Self>;
}

impl Loadable for PlayerControl {}
impl ComponentLoadable for PlayerControl {}

impl PlayerControl {
    pub fn load(_ecs: Arc<RwLock<World>>, _window: Arc<RwLock<&mut Window>>, _json_value: Value) -> Task<ComponentType> {
        Task::new( || {
            Ok(ComponentType::PlayerControl(PlayerControl{}))
        })
    }
}
