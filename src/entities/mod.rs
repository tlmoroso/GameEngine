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
use tracing::{instrument, trace, error};
use specs::world::EntitiesRes;
use crate::loading::DrawTask;
use luminance_glfw::GL33Context;
use std::borrow::BorrowMut;

pub mod player;
pub mod textbox;

pub const ENTITIES_DIR: &str = "entities/";
pub const ENTITY_LOADER_FILE_ID: &str = "entity_loader";

#[derive(Deserialize, Debug)]
pub(crate) struct EntityLoaderJSON {
    component_paths: Vec<String>
}

pub struct EntityLoader {
    entity_file: String,
}

impl EntityLoader {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: EntityLoader::new");

        let new = Self {
            entity_file: file_path,
        };

        #[cfg(feature="trace")]
        trace!("EXIT: EntityLoader::new");

        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs, window)))]
    pub fn load_entity<T: ComponentMux>(&self) -> DrawTask<Entity> {
        let file_path = self.entity_file.clone();    // Attempt to not have self in the closure

        DrawTask::new(move |(world, context)| {
            let entity_json: EntityLoaderJSON = load_deserializable_from_file(&file_path, ENTITY_LOADER_FILE_ID)?;

            let ecs = world.read().expect("Failed to lock World");

            let lazy_update = ecs.fetch::<LazyUpdate>();
            let entities = ecs.fetch::<EntitiesRes>();

            let mut builder = lazy_update.create_entity(&entities);

            for component_path in entity_json.component_paths {
                eprintln!("Loading component: {:?}", component_path.clone());
                let json = load_json(&component_path)?;
                let loader = T::map_json_to_loader(json)?;

                builder = loader.load_component(builder, world.clone(), Some(context.clone()))?;
                eprintln!("component loaded");
            }

            Ok(builder.build())
        })
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