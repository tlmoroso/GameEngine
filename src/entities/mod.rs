use coffee::graphics::Window;
use coffee::load::{Task};

use specs::{Builder, Entity, World, WorldExt, LazyUpdate};

use serde::Deserialize;

use std::io::ErrorKind;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use crate::components::{ComponentLoader, ComponentMux};
use crate::load::{load_json, LoadError, build_task_error, load_deserializable_from_file};
use crate::entities::EntityError::{EntityFileLoadError, EntityComponentLoaderError, EntityLoadComponentError, EntityLoaderDeserializeError, EntityWorldRWLockError};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, trace, error};
use specs::world::EntitiesRes;

pub mod player;
pub mod textbox;

pub const ENTITIES_DIR: &str = "entities/";
pub const ENTITY_LOADER_FILE_ID: &str = "entity_loader";

#[derive(Deserialize, Debug)]
pub(crate) struct EntityLoaderJSON {
    component_paths: Vec<String>
}

pub struct EntityLoader<T: ComponentMux> {
    entity_file: String,
    component_loaders: Vec<Box<dyn ComponentLoader>>,
    phantom: PhantomData<T>,
}

impl<T: ComponentMux> EntityLoader<T> {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: EntityLoader::new");

        let new = Self {
            entity_file: file_path,
            component_loaders: Vec::new(),
            phantom: PhantomData,
        };

        #[cfg(feature="trace")]
        trace!("EXIT: EntityLoader::new");

        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs, window)))]
    pub fn load_entity(&mut self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<Entity> {
        #[cfg(feature="trace")]
        trace!("ENTER: EntityLoader::load_entity");

        let entity_loader_json: EntityLoaderJSON = map_err_return!(
            load_deserializable_from_file(&self.entity_file, ENTITY_LOADER_FILE_ID),
            |e| {
                build_task_error(
                    EntityLoaderDeserializeError {
                        file_path: self.entity_file.to_string(),
                        source: e
                    },
                    ErrorKind::InvalidData
                )
            }
        );
        
        for component_path in entity_loader_json.component_paths {
            println!("Component Path: {:?}", component_path);
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

            #[cfg(feature="trace")]
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
        let read_ecs = map_err_return!(
            ecs.read(),
            |e| {
                build_task_error(
                    EntityWorldRWLockError {
                        var_name: stringify!(ecs).to_string(),
                        source_string: format!("{}", e)
                    },
                    ErrorKind::NotFound
                )
            }
        );

        #[cfg(feature="trace")]
        trace!("Successfully grabbed read lock for World");

        let entities = read_ecs.fetch::<EntitiesRes>();
        let lazy_update = read_ecs.fetch::<LazyUpdate>();
        let mut builder = lazy_update.create_entity(&entities);

        for component_loader in &self.component_loaders {
            builder = map_err_return!(
                component_loader.load_component(builder, &read_ecs, window),
                |e| {
                    build_task_error(
                        EntityLoadComponentError {
                            source: e
                        },
                        ErrorKind::InvalidData
                    )
                }
            );

            #[cfg(feature="trace")]
            trace!("Added: {} to entity", component_loader.get_component_name());
        }

        let entity = builder.build();

        #[cfg(feature="trace")]
        trace!("Entity: {:#?} built", entity);

        #[cfg(feature="trace")]
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
    #[error("Error creating EntityLoader from: {file_path}")]
    EntityLoaderDeserializeError {
        file_path: String,
        source: LoadError
    },
    #[error("Error creating component loader from path: {component_path}")]
    EntityComponentLoaderError {
        component_path: String,
        source: anyhow::Error
    },
    #[error("Error retrieving lock for {var_name}\nError Source: {source_string}")]
    EntityWorldRWLockError {
        var_name: String,
        source_string: String
    },
    #[error("Error loading component from Component Loader")]
    EntityLoadComponentError {
        source: anyhow::Error
    }
}