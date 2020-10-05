use coffee::graphics::Window;
use coffee::load::{Task};
use specs::{Builder, Entity, World, WorldExt};

use crate::load::{Loadable, load_json};
use serde_json::{Value, from_value};
use serde::Deserialize;
use crate::components::animation::{Animation, ANIMATION_FILE_ID};
use crate::components::mesh_graphic::{MeshGraphic, MESH_GRAPHIC_FILE_ID};
use crate::components::player_control::{PlayerControl, PLAYER_CONTROL_FILE_ID};
use crate::components::position::{Position, POSITION_FILE_ID};
use crate::components::text_display::{TextDisplay, TEXT_DISPLAY_FILE_ID};
use crate::components::ComponentType;
use std::sync::{Arc, RwLock};

pub mod player;
pub mod textbox;

pub const LOADABLE_ENTITY_FILE_ID: &str = "entity";

#[derive(Deserialize, Debug)]
struct ComponentVecJSON(Vec<String>);

pub struct LoadableEntity {
    pub entity: Entity
}


impl Loadable for LoadableEntity {}

impl LoadableEntity {
    pub(crate) fn load(ecs: Arc<RwLock<World>>, window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<Self>  {
        let component_vec: ComponentVecJSON = from_value(json_value)
            .expect("ERROR: could not translate JSON to component_vec in Entity::load");
        let mut components = Vec::new();

        for component_path in component_vec.0 {
            let json_value = load_json(component_path);
            let component_task =
                match json_value.loadable_type.as_str() {
                    ANIMATION_FILE_ID => Animation::load(ecs.clone(), window.clone(), json_value.other_value),
                    MESH_GRAPHIC_FILE_ID => MeshGraphic::load(ecs.clone(), window.clone(), json_value.other_value),
                    PLAYER_CONTROL_FILE_ID => PlayerControl::load(ecs.clone(), window.clone(), json_value.other_value),
                    POSITION_FILE_ID => Position::load(ecs.clone(), window.clone(), json_value.other_value),
                    TEXT_DISPLAY_FILE_ID => TextDisplay::load(ecs.clone(), window.clone(), json_value.other_value),
                    _ => panic!(format!("ERROR: component loadable type invalid: {}", json_value.loadable_type)),
                };
            components.push(component_task
                .run(window.write().expect("ERROR: RwLock poisoned for window in LoadableEntity::load").gpu())
                .expect("ERROR: Failed to run component task in LoadableEntity::load"));
        }

        // Must wait until after all components are loaded to grab lock on ecs, so components can access lock.
        let mut world_mut = ecs
            .write()
            .expect("ERROR: RwLock guard poisoned in LoadableEntity::load");
        let mut entity_builder = world_mut
            .create_entity();

        for component in components {
            entity_builder = match component {
            ComponentType::Animation(animation) => entity_builder.with(animation),
                ComponentType::MeshGraphic(mesh_graphic) => entity_builder.with(mesh_graphic),
                ComponentType::PlayerControl(player_control) => entity_builder.with(player_control),
                ComponentType::Position(position) => entity_builder.with(position),
                ComponentType::TextDisplay(text_display) => entity_builder.with(text_display),
            };
        }

        let entity = entity_builder.build();
        println!("LoadableEntity::load complete");
        Task::new(move || {
            Ok(
                LoadableEntity {
                    entity
                }
            )
        })
    }
}