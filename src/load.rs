use serde_json::{Value, from_str, from_value};
use serde::Deserialize;

use std::io::ErrorKind;
use std::fs::read_to_string;
use std::error::Error;

use thiserror::Error;

use crate::load::LoadError::{JSONLoadConversionError, ValueConversionError, ReadError};

use coffee::load::Task;

#[cfg(trace)]
use tracing::{instrument, trace, debug};

pub const LOAD_PATH: &str = "assets/JSON/";
pub const JSON_FILE: &str = ".json";

#[macro_export]
macro_rules! map_err_return {
    ( $e:expr, $err:expr ) => {
        match $e {
            Ok(x) => x,
            Err(e) => return $err(e)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct JSONLoad {
    pub load_type_id: String,
    pub actual_value: Value
}

#[cfg_attr(trace, instrument(err))]
pub fn load_json(file_path: &str) -> Result<JSONLoad, LoadError> {
    #[cfg(trace)]
    trace!("ENTER: load_json");
    let json_string = read_to_string(file_path).map_err(|e| { ReadError {path: file_path.to_string(), source: e} })?;
    #[cfg(trace)]
    debug!("Successfully loaded file into string from: {}", file_path);
    let json_value = from_str::<Value>(json_string.as_str()).map_err(|e| { ValueConversionError {string_value: json_string, source: e} })?;
    let load_json = from_value(json_value.clone()).map_err(|e| { JSONLoadConversionError { value: json_value, source: e } });
    #[cfg(trace)]
    trace!("EXIT: load_json");
    return load_json
}

#[cfg_attr(trace, instrument)]
pub fn build_task_error<T>(error: impl Error + Sync + Send + 'static, error_kind: ErrorKind) -> Task<T> {
    #[cfg(trace)]
    trace!("ENTER: build_task_error");
    let task = Task::new(move || { Err(
        coffee::Error::IO(std::io::Error::new(error_kind, anyhow::Error::new(error)))
    )});
    #[cfg(trace)]
    trace!("EXIT: build_task_error");
    return task
}

// pub fn convert<T, E: Error>(value: Value, kind: ErrorKind) -> Result<T, >

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
    #[error("Error matching given load ID to type expected.\nFrom: {json_path}\nExpected: {expected}\nActual: {actual}")]
    LoadIDError {
        actual: String,
        expected: String,
        json_path: String,
    }
}