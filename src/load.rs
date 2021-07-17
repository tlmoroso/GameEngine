use serde_json::{Value, from_str, from_value};
use serde::Deserialize;

use std::fs::read_to_string;
use std::error::Error;
use std::sync::{RwLock, Arc};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, trace, debug};

use specs::{World, Entity};

use crate::entities::{EntityLoader};
use crate::load::LoadError::{JSONLoadConversionError, ValueConversionError, ReadError, LoadIDError, DeserializationError};
use crate::components::ComponentMux;
use std::fmt::Debug;
use crate::loading::{Task, DrawTask};

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
            ReadError {
                path: file_path.to_string(), source: e
            }
        })?;

    #[cfg(feature="trace")]
    debug!("Successfully loaded file into string from: {}", file_path);

    let json_value = from_str::<Value>(json_string.as_str())
        .map_err(|e| {
            ValueConversionError {
                string_value: json_string,
                source: e
            }
        })?;

    let load_json = from_value(json_value.clone())
        .map_err(|e| {
            JSONLoadConversionError {
                value: json_value,
                source: e
            }
        });

    #[cfg(feature="trace")]
    trace!("EXIT: load_json");

    return load_json;
}

// #[cfg_attr(feature="trace", instrument)]
// pub fn build_task_error<T: 'static>(error: impl Error + Send + Sync + 'static) -> Task<T> {
//     #[cfg(feature="trace")]
//     trace!("ENTER: build_task_error");
//
//     let task = Task::new(|_| { Err(
//         // coffee::Error::IO(std::io::Error::new(error_kind, error))
//         anyhow::Error::new(error)
//     )});
//
//     #[cfg(feature="trace")]
//     trace!("EXIT: build_task_error");
//
//     return task
// }

#[cfg_attr(feature="trace", instrument(skip(ecs, window)))]
pub fn load_entity_vec<T: 'static + ComponentMux>(entity_paths: &Vec<String>) -> DrawTask<Vec<Entity>> {
    let mut entity_task = DrawTask::new(|(_ecs, _context)| {Ok(
        Vec::new()
    )});

    for entity_path in entity_paths {
        eprintln!("Loading entity: {:?}", entity_path.clone());
        let path = entity_path.clone();
        let entity_loader = EntityLoader::new(path);
        let other = entity_loader.load_entity::<T>();
        entity_task = entity_task.join(other, |(mut entity_vec, entity)| {
            entity_vec.push(entity);
            entity_vec
        });
        eprintln!("entity loaded");
    }

    return entity_task
}

#[cfg_attr(feature="trace", instrument)]
pub fn load_deserializable_from_file<T: for<'de> Deserialize<'de>>(file_path: &str, file_id: &str) -> Result<T, LoadError> {
    #[cfg(feature="trace")]
    trace!("ENTER: load_deserializable_from_file");

    let json_value = load_json(file_path)?;

    #[cfg(feature="trace")]
    trace!("Successfully loaded JSONLoad: {:#?} from: {:#?}", json_value, file_path.to_string());

    if json_value.load_type_id != file_id {
        return Err( LoadIDError {
                actual: json_value.load_type_id,
                expected: file_id.to_string(),
            })
    }

    #[cfg(feature="trace")]
    trace!("Load ID: {} matched given file ID", json_value.load_type_id);

    let deserialized_value: Result<T, LoadError> = from_value(json_value.actual_value.clone())
        .map_err(|e| {
            DeserializationError {
                value: json_value.actual_value,
                source: e
            }
        });

    #[cfg(feature="trace")]
    trace!("EXIT: load_deserializable_from_file");

    return deserialized_value
}

#[cfg_attr(feature="trace", instrument)]
pub fn load_deserializable_from_json<T: for<'de> Deserialize<'de>>(json: &JSONLoad, load_id: &str) -> Result<T, LoadError> {
    return if json.load_type_id == load_id {
        from_value::<T>(json.actual_value.clone())
            .map_err(|e| {
                JSONLoadConversionError {
                    value: json.actual_value.clone(),
                    source: e
                }
            })
    } else {
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