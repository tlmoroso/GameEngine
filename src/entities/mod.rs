use coffee::graphics::Window;
use coffee::load::{Task};

use specs::{Builder, Entity, World, WorldExt};

use serde_json::{Value, from_value};
use serde::Deserialize;

use std::io::ErrorKind;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use crate::components::{ComponentLoader, ComponentMux};
use crate::load::{load_json, LoadError, build_task_error};
use crate::entities::EntityError::{EntityFileLoadError, EntityJSONLoadError, EntityComponentLoaderError, EntityWorldWriteLockError, EntityLoadComponentError};
use crate::load::LoadError::LoadIDError;

use thiserror::Error;

#[cfg(trace)]
use tracing::{instrument, trace, error};

pub mod player;
pub mod textbox;

pub const ENTITIES_DIR: &str = "entities/";
pub const ENTITY_LOADER_FILE_ID: &str = "entity_loader";

#[derive(Deserialize, Debug)]
struct EntityLoaderJSON {
    component_paths: Vec<String>
}

pub struct EntityLoader<T: ComponentMux> {
    entity_file: String,
    component_loaders: Vec<Box<dyn ComponentLoader>>,
    phantom: PhantomData<T>,
}

impl<T: ComponentMux> EntityLoader<T> {
    #[cfg_attr(trace, instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(trace)]
trace!("ENTER: EntityLoader::new");
        let new = Self {
            entity_file: file_path,
            component_loaders: Vec::new(),
            phantom: PhantomData,
        };
        #[cfg(trace)]
trace!("EXIT: EntityLoader::new");
        return new
    }

    #[cfg_attr(trace, instrument(skip(self, ecs, window)))]
    pub fn load_entity(&mut self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<Entity> {
        #[cfg(trace)]
trace!("ENTER: EntityLoader::load_entity");
        let json_value = map_err_return!(
            load_json(&self.entity_file),
            |e| {
                build_task_error(
                    EntityFileLoadError {
                        file: self.entity_file.clone(),
                        var_name: stringify!(self.entity_file).to_string(),
                        source: e
                    },
                 ErrorKind::InvalidData
                )
            }
        );
        #[cfg(trace)]
trace!("Successfully loaded JSONLoad: {} from: {}", json_value, self.entity_file);

        if json_value.load_type_id != ENTITY_LOADER_FILE_ID {
            return build_task_error(
                LoadIDError {
                    actual: json_value.load_type_id,
                    expected: ENTITY_LOADER_FILE_ID.to_owned(),
                    json_path: self.entity_file.to_owned()
                },
                ErrorKind::InvalidData
            )
        }
        #[cfg(trace)]
trace!("Load ID: {} matched ENTITY_LOADER_FILE_ID", json_value.load_type_id);

        let component_paths: EntityLoaderJSON = map_err_return!(
            from_value(json_value.actual_value.clone()),
            |e| {
                build_task_error(
                    EntityJSONLoadError {
                        value: json_value.actual_value.clone(),
                        source: e
                    },
                    ErrorKind::InvalidData
                )
            }
        );
        #[cfg(trace)]
trace!("EntityLoaderJSON: {:#?} successfully loaded from {:#?}", json_value.actual_value);

        for component_path in component_paths.component_paths {
            let json_value = map_err_return!(
                load_json(&component_path),
                |e| {
                    build_task_error(
                        EntityFileLoadError {
                            file: component_path.clone(),
                            var_name: stringify!(component_path).to_string(),
                            source: e
                        },
                        ErrorKind::InvalidData
                    )
                }
            );
            #[cfg(trace)]
trace!("Value: {:#?} loaded from: {:#?}", json_value, component_path);

            self.component_loaders.push( map_err_return!(
                T::map_json_to_loader(json_value),
                |e| {
                    build_task_error(
                        EntityComponentLoaderError {
                                component_path,
                                source: e
                            },
                        ErrorKind::InvalidData
                    )
                }
            ));
        }
        // Must clone to appease compiler, so mutable ref to ecs is not dropped before the entity is built.
        let mut mut_ecs = map_err_return!(
            ecs.write(),
            |e| {
                build_task_error(
                    EntityWorldWriteLockError {
                        var_name: stringify!(ecs).to_string(),
                        source_string: format!("{}", e)
                    },
                    ErrorKind::NotFound
                )
            }
        );
        #[cfg(trace)]
trace!("Successfully grabbed write lock for World");

        let mut entity_builder = mut_ecs.create_entity();
        for component_loader in &self.component_loaders {
            entity_builder = map_err_return!(
                component_loader.load_component(entity_builder, ecs.clone(), window),
                |e| {
                    build_task_error(
                        EntityLoadComponentError {
                            source: e
                        },
                        ErrorKind::InvalidData
                    )
                }
            );
            #[cfg(trace)]
trace!("Added: {} to entity", component_loader.get_name());
        }

        let entity = entity_builder.build();
        #[cfg(trace)]
trace!("Entity: {:#?} built", entity);
        #[cfg(trace)]
trace!("EXIT: EntityLoader::load_entity");
        Task::new(move || { Ok(
            entity
        )})
    }
}

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("Error loading JSON Value from: {var_name} = {file}")]
    EntityFileLoadError {
        file: String,
        var_name: String,
        source: LoadError
    },
    #[error("Error converting serde_json::value::Value into EntityLoaderJSON.\nExpected: Value::Array<Value::String>\nActual: {value}")]
    EntityJSONLoadError {
        value: Value,
        source: serde_json::error::Error
    },
    #[error("Error creating component loader from path: {component_path}")]
    EntityComponentLoaderError {
        component_path: String,
        source: anyhow::Error
    },
    #[error("Error retrieving lock for {var_name}\nError Source: {source_string}")]
    EntityWorldWriteLockError {
        var_name: String,
        source_string: String
    },
    #[error("Error loading component from Component Loader")]
    EntityLoadComponentError {
        source: anyhow::Error
    }
}