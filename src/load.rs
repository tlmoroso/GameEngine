use serde_json::{Value, from_str, from_value};
use serde::Deserialize;

use std::fs::read_to_string;
use std::error::Error;
use std::sync::{RwLock, Arc};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, trace, debug, error};

use specs::{World, Entity};

use crate::entities::{EntityLoader};
use crate::load::LoadError::{JSONLoadConversionError, ValueConversionError, ReadError, LoadIDError, DeserializationError, ExecutionError};
use crate::components::ComponentMux;
use std::fmt::Debug;
use crate::loading::{Task, DrawTask};
use luminance_glfw::GL33Context;

pub const LOAD_PATH: &str = "assets/JSON/";
pub const JSON_FILE: &str = ".json";

pub const ENTITY_VEC_LOAD_ID: &str = "entity_vec";

#[macro_export]
macro_rules! map_err_return {
    ( $e:expr, $err:expr ) => {
        match $e {
            Ok(x) => x,
            Err(e) => return $err(e)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct JSONLoad {
    pub load_type_id: String,
    pub actual_value: Value
}

#[cfg_attr(feature="trace", instrument)]
pub fn load_json(file_path: &str) -> Result<JSONLoad, LoadError> {
    #[cfg(feature="trace")]
    trace!("ENTER: load_json");

    let json_string = read_to_string(file_path)
        .map_err(|e| {
            #[cfg(feature = "trace")]
            error!("Something went wrong while reading in json from file: {:?}", file_path.clone());

            ReadError {
                path: file_path.to_string(), source: e
            }
        })?;

    #[cfg(feature="trace")]
    debug!("Successfully loaded file into string from: {:?}", file_path.clone());

    let json_value = from_str::<Value>(json_string.as_str())
        .map_err(|e| {
            #[cfg(feature = "trace")]
            error!("Error converting json string: ({:?}) into serde_json Value.", json_string.clone());

            ValueConversionError {
                string_value: json_string.clone(),
                source: e
            }
        })?;

    #[cfg(feature = "trace")]
    debug!("JSON string: ({:?}) from file translated into serde_json value: {:?}", json_string.clone(), json_value.clone());

    let load_json = from_value(json_value.clone())
        .map_err(|e| {
            #[cfg(feature = "trace")]
            error!("Error occurred while converting serde_json Value into JSONLoad object");

            JSONLoadConversionError {
                value: json_value,
                source: e
            }
        });

    #[cfg(feature="trace")]
    debug!("EXIT: load_json. value: {:?}", load_json);

    return load_json;
}

#[cfg_attr(feature="trace", instrument(skip(ecs, context)))]
pub fn create_entity_vec<T: 'static + ComponentMux>(entity_paths: &Vec<String>, ecs: Arc<RwLock<World>>, context: Arc<RwLock<GL33Context>>) -> Result<Vec<Entity>, LoadError> {
    let mut entity_vec = Vec::new();

    for entity_path in entity_paths {
        #[cfg(feature = "trace")]
        debug!("Loading entity from: {:?}", entity_path.clone());

        let entity = EntityLoader::new(entity_path.clone())
            .load_entity::<T>()
            .execute((ecs.clone(), context.clone()))
            .map_err(|e| {
                #[cfg(feature = "trace")]
                debug!("A failure occurred during execution of the entity task");

                ExecutionError {
                    source: e
                }
            })?;
        #[cfg(feature = "trace")]
        debug!("Entity loaded");

        entity_vec.push(entity);
        #[cfg(feature = "trace")]
        debug!("Entity appended to vec");
    }

    return Ok(entity_vec)
}

#[cfg_attr(feature="trace", instrument)]
pub fn load_deserializable_from_file<T: for<'de> Deserialize<'de> + Debug>(file_path: &str, load_id: &str) -> Result<T, LoadError> {
    let json_value = load_json(file_path)
        .map_err(|e| {
            #[cfg(feature = "trace")]
            error!("Something went wrong while loading a JSONLoad object from file. Path: ({:?}). ID: {:?}", file_path.clone(), load_id.clone());

            return e
        })?;

    #[cfg(feature="trace")]
    debug!("Successfully loaded JSONLoad: ({:?}) from: {:?}", json_value.clone(), file_path.clone());

    if json_value.load_type_id != load_id {
        #[cfg(feature = "trace")]
        error!("Type ID: ({:?}) of loaded object does not match given type ID: {:?}", json_value.load_type_id.clone(), load_id.clone());

        return Err( LoadIDError {
                actual: json_value.load_type_id,
                expected: load_id.to_string(),
            })
    }

    #[cfg(feature="trace")]
    debug!("Load ID: ({:?}) matched given file ID: {:?}", json_value.load_type_id.clone(), load_id.clone());

    let deserialized_value: Result<T, LoadError> = from_value(json_value.actual_value.clone())
        .map_err(|e| {
            #[cfg(feature = "trace")]
            error!("Failed to convert generic JSONLoad object: ({:?}) into specific type", json_value.clone());

            DeserializationError {
                value: json_value.actual_value,
                source: e
            }
        });

    return deserialized_value
}

#[cfg_attr(feature="trace", instrument)]
pub fn load_deserializable_from_json<T: for<'de> Deserialize<'de>>(json: &JSONLoad, load_id: &str) -> Result<T, LoadError> {
    return if json.load_type_id == load_id {
        from_value::<T>(json.actual_value.clone())
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to convert json load object: ({:?}) into given type", json.clone());

                JSONLoadConversionError {
                    value: json.actual_value.clone(),
                    source: e
                }
            })
    } else {
        #[cfg(feature = "trace")]
        error!("Given load_id: ({:?}) did not match load_id of json object: {:?}", load_id.clone(), json.load_type_id.clone());

        Err(
            LoadIDError {
                actual: json.load_type_id.clone(),
                expected: load_id.to_string()
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Error loading file at path: {path}")]
    ReadError {
        path: String,
        source: std::io::Error
    },
    #[error("Error creating serde_json::Value at (line: {:#?}, column: {:#?}) of type: {:#?} from file string: {string_value}", .source.line(), .source.column(), source.classify())]
    ValueConversionError {
        string_value: String,
        source: serde_json::error::Error
    },
    #[error("Error creating load::JSONLoad from serde_json::value::Value. \nExpected: {{\"load_type_id\": String, \"actual_value\": Object}} \nGot: {value}")]
    JSONLoadConversionError {
        value: Value,
        source: serde_json::error::Error
    },
    #[error("Error matching given load ID to type expected.\nExpected: {expected}\nActual: {actual}")]
    LoadIDError {
        actual: String,
        expected: String,
    },
    #[error("Error deserializing serde_json::Value: {value}")]
    DeserializationError {
        value: Value,
        source: serde_json::error::Error
    },
    #[error("Failed to execute task")]
    ExecutionError {
        source: anyhow::Error
    }
}

#[derive(Debug, Error)]
pub enum LoadActionError {
    #[error("Failed to convert serde_json::Value: {json_value} into Vec<String>")]
    PathVecConversionError {
        json_value: Value,
        source: LoadError
    },
    #[error("Failed to create JSONLoad object from file path: {file_path}")]
    JSONLoadError {
        file_path: String,
        source: LoadError
    }
}