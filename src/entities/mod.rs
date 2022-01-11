// use coffee::load::{Task};

use specs::{Builder, Entity, LazyUpdate, World};

use serde::Deserialize;

use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use crate::components::{ComponentLoader, ComponentMux};
use crate::load::{load_json, LoadError, load_deserializable_from_file, JSONLoad};
// use crate::entities::EntityError::{EntityFileLoadError, EntityComponentLoaderError, EntityLoadComponentError, EntityLoaderDeserializeError};

use thiserror::Error;

use anyhow::Result;

#[cfg(feature="trace")]
use tracing::{instrument, error, debug};
use specs::world::EntitiesRes;
use crate::loading::{DrawTask, GenTask};
use luminance_glfw::GL33Context;
use std::borrow::BorrowMut;
use crate::entities::EntityError::{EntityLoaderDeserializeError, EntityWorldWriteLockError, EntityFileLoadError, ComponentMuxError, EntityComponentLoaderError};

pub mod player;
pub mod textbox;

pub const ENTITIES_DIR: &str = "entities/";
pub const ENTITY_LOAD_ID: &str = "entity_loader";

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EntityLoaderJSON {
    component_paths: Vec<String>
}

#[derive(Debug, Clone)]
pub struct EntityLoader {
    entity_file: String,
}

impl EntityLoader {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        let new = Self {
            entity_file: file_path,
        };

        #[cfg(feature = "trace")]
        debug!("Successfully created new EntityLoader from given path: {:?}", new);

        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self)))]
    pub fn load_entity<T: ComponentMux>(&self) -> GenTask<Entity> {
        let file_path = self.entity_file.clone();    // Attempt to not have self in the closure

        GenTask::new(move |ecs| {
            let entity_json: EntityLoaderJSON = load_deserializable_from_file(&file_path, ENTITY_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to load JSON value for Entity from file: {:?}", file_path.clone());

                    EntityLoaderDeserializeError {
                        source: e,
                        file_path: file_path.clone()
                    }
                })?;

            #[cfg(feature = "trace")]
            debug!("Entity JSON value loaded from file: {:?}", file_path.clone());

            let ecs_read = ecs.read()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    debug!("Error acquiring write lock for World");

                    EntityWorldWriteLockError
                })?;

            let lazy_update = ecs_read.fetch::<LazyUpdate>();
            let entities = ecs_read.fetch::<EntitiesRes>();

            let mut builder = lazy_update.create_entity(&entities);

            #[cfg(feature = "trace")]
            debug!("Lazy Builder has been created for building Entity");

            for component_path in entity_json.component_paths {
                #[cfg(feature = "trace")]
                debug!("Loading component from: {:?}", component_path.clone());
                let json = load_json(&component_path)
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("Error occurred while loading component JSON value.");

                        EntityFileLoadError {
                            file: component_path.clone(),
                            source: e
                        }
                    })?;
                let loader = T::map_json_to_loader(json.clone())
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("Error occurred while mapping JSON value: ({:?}) to Component type", json);

                        ComponentMuxError {
                            source: e,
                            component_json: json
                        }
                    })?;

                builder = loader.load_component(builder, ecs.clone())
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("Error occurred while loading component.");

                        EntityComponentLoaderError {
                            component_path,
                            source: e
                        }
                    })?;
                #[cfg(feature = "trace")]
                debug!("Component loaded");
            }

            let entity = builder.build();

            #[cfg(feature = "trace")]
            debug!("Entity built: {:?}", entity);

            return Ok(entity)
        })
    }
}

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("Error loading JSON Value from {file}")]
    EntityFileLoadError {
        file: String,
        source: LoadError
    },
    #[error("Error creating EntityLoader JSON from: {file_path}")]
    EntityLoaderDeserializeError {
        file_path: String,
        source: LoadError
    },
    #[error("Error creating component loader from path: {component_path}")]
    EntityComponentLoaderError {
        component_path: String,
        source: anyhow::Error
    },
    #[error("Error retrieving write lock for World")]
    EntityWorldWriteLockError,
    #[error("Error loading component from Component Loader")]
    EntityLoadComponentError {
        source: anyhow::Error
    },
    #[error("Error matching component JSON value {component_json:?} to Component")]
    ComponentMuxError {
        source: anyhow::Error,
        component_json: JSONLoad
    }
}