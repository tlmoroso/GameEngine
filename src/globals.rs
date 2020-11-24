use crate::globals::FontDictError::{FontDictFileLoadError, FontDictJSONLoadError, FontDictFileReadError, FontDictFontSizeError};
use crate::load::{load_json, LoadError, build_task_error};

use std::collections::{HashMap};
use std::fs;
use std::sync::{Arc, RwLock};
use std::io::ErrorKind;

use coffee::graphics::{Image, Font, Window};
use coffee::load::{Task, Join};

use specs::{World};

use serde_json::{Value, from_value};
use serde::Deserialize;

use thiserror::Error;

pub const FONT_DICT_FILE_ID: &str = "font_dict";

pub struct ImageDict(pub(crate) HashMap<String, Image>);
pub struct FontDict(pub(crate) HashMap<String, Font>);

pub const FONTS_DIR: &str = "fonts/";

const FONT_VEC_SIZE: usize = 4;
const FONT_FILE_SIZE: usize = 60_000;
static mut FONT_BYTES: [[u8; FONT_FILE_SIZE]; FONT_VEC_SIZE] = [[0; FONT_FILE_SIZE]; FONT_VEC_SIZE];

unsafe impl Send for FontDict {}
unsafe impl Sync for FontDict {}

#[derive(Deserialize)]
pub struct FontDictLoader {
    path: String
}

#[derive(Deserialize)]
struct FontDictLoaderJSON {
    fonts: HashMap<String, String>
}

impl FontDictLoader {
    pub fn new(file_path: String) -> Self {
        Self {
            path: file_path
        }
    }

    pub fn load(self, _ecs: Arc<RwLock<World>>, _window: &Window) -> Task<FontDict> {
        let mut font_task = Task::new(|| { Ok(
            HashMap::new()
        )});

        let json_value = map_err_return!(
            load_json(&self.path),
            |e| { build_task_error(
                FontDictFileLoadError {
                    path: self.path,
                    var_name: stringify!(self.path).to_string(),
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );

        if json_value.load_type_id == FONT_DICT_FILE_ID {
            return build_task_error(
                LoadError::LoadIDError {
                    actual: json_value.load_type_id,
                    expected: FONT_DICT_FILE_ID.to_string(),
                    json_path: self.path.clone()
                },
                ErrorKind::InvalidData
            )
        }

        let fonts: FontDictLoaderJSON = map_err_return!(
            from_value(json_value.actual_value.clone()),
            |e| { build_task_error(
                FontDictJSONLoadError {
                    value: json_value.actual_value,
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );

        for (index, (font_name, font_path)) in fonts.fonts.into_iter().enumerate() {
            let font = map_err_return!(
                fs::read(font_path.clone()),
                |e| { build_task_error(
                    FontDictFileReadError {
                        path: font_path,
                        source: e
                    },
                    ErrorKind::InvalidData
                )}
            );

            if font.len() <= FONT_VEC_SIZE {
                unsafe {
                    for (i, byte) in font.iter().enumerate() {
                        FONT_BYTES[index][i] = *byte
                    }
                };
            } else {
                return build_task_error(
                    FontDictFontSizeError {
                        font_size: font.len(),
                        font_name,
                        font_path: font_path.clone()
                    },
                    ErrorKind::InvalidData
                )
            }


            font_task = (
                Font::load_from_bytes(unsafe { &FONT_BYTES[index] }),
                font_task
            )
                .join()
                .map(|(font, mut font_dict)| {
                    font_dict.insert(font_name, font);
                    return font_dict
                })
        }

        font_task.map(|font_dict| {
            FontDict(font_dict)
        })
    }
}

#[derive(Error, Debug)]
pub enum FontDictError {
    #[error("Error loading JSON Value for FontDictLoader from: {var_name} = {path}")]
    FontDictFileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    },
    #[error("Error converting serde_json::value::Value into FontDictLoaderJSON.\nExpected: Value::Object<Value::String, Value::String>\nActual: {value}")]
    FontDictJSONLoadError {
        value: Value,
        source: serde_json::error::Error
    },
    #[error("Error reading file at {path}")]
    FontDictFileReadError {
        path: String,
        source: std::io::Error
    },
    #[error("Error: {font_name} at {font_path} has size {font_size} greater than {} byte limit")]
    FontDictFontSizeError {
        font_size: usize,
        font_name: String,
        font_path: String
    }
}