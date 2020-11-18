use coffee::graphics::Window;
use coffee::load::{Task};

use specs::{Builder, Entity, World, WorldExt};

use serde_json::{Value};

use std::io::ErrorKind;
use std::marker::PhantomData;

use crate::components::{ComponentLoader, ComponentMux};
use crate::load::{load_json};

pub mod player;
pub mod textbox;

pub const ENTITIES_DIR: &str = "entities/";


pub struct EntityLoader<T: ComponentMux> {
    entity_file: String,
    phantom: PhantomData<T>,
}

impl<T: ComponentMux> EntityLoader<T> {
    pub fn new(file_path: String) -> Self {
        Self {
            entity_file: file_path,
            phantom: PhantomData,
        }
    }

    pub fn load_entity(&self, ecs: &'static mut World, window: &Window) -> Task<Entity> {
        let json_value = load_json(&self.entity_file).unwrap();
        return if let Value::Array(component_paths) = json_value.actual_value {
            let mut entity_task = Task::new(|| {Ok(
                ecs.create_entity()
            )});

            for component_path in component_paths {
                if let Value::String(component_path) = component_path {
                    let json_value = load_json(&component_path).unwrap();
                    let component_loader = T::map_id_to_loader(json_value.load_type_id);
                    entity_task = component_loader.load_component(json_value.actual_value, entity_task, ecs,window);
                } else {
                    return Task::new(|| { Err(
                        coffee::Error::IO(std::io::Error::new(ErrorKind::InvalidData, "ERROR: expected string describing component path"))
                    )})
                }
            }

            entity_task.map(|entity_builder| {
                entity_builder.build()
            })
        } else {
            Task::new(|| { Err(
                coffee::Error::IO(std::io::Error::new(ErrorKind::InvalidData, "ERROR: expected Array of path strings"))
            )})
        }
    }
}