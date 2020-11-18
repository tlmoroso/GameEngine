use std::collections::{HashMap};
use std::fs;
use std::io::ErrorKind;

use crate::load::{load_json};

use coffee::graphics::{Image, Font, Window};
use coffee::load::{Task, Join};
use coffee::Error;

use specs::{World};

use serde_json::{Value};
use serde::Deserialize;

pub const FONT_DICT_FILE_ID: &str = "font_dict";

#[derive(Deserialize, Debug)]
struct FontMapJSON(HashMap<String, String>);

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

impl FontDictLoader {
    pub fn new(file_path: String) -> Self {
        Self {
            path: file_path
        }
    }

    fn load(self, _ecs: &mut World, _window: &Window) -> Task<FontDict> {
        let mut font_task = Task::new(|| { Ok(
            HashMap::new()
        )});

        let json_value = load_json(&self.path).unwrap();
        return if let Value::Object(loadable_fonts) = json_value.actual_value {
            for (index, (font_name, font_path)) in loadable_fonts.into_iter().enumerate() {
                if let Value::String(font_path) = font_path {
                    let font_name= font_name.clone();
                    let font_path = font_path.clone();

                    let font = fs::read(font_path).unwrap();
                    if font.len() <= FONT_VEC_SIZE {
                        unsafe {
                            font.iter().enumerate().map(|(i, byte)| { FONT_BYTES[index][i] = *byte } )
                        };
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
                } else {
                    return Task::new(move || {
                        coffee::Result::Err(
                            Error::IO(std::io::Error::new(ErrorKind::InvalidData, format!("Incorrect JSON Value: Expected Value::String, Got {:?}", font_path)))
                        )
                    })
                }
            }
            font_task.map(|font_dict| {
                FontDict(font_dict)
            })
        } else {
            Task::new(move || {
                coffee::Result::Err(
                    Error::IO(std::io::Error::new(ErrorKind::InvalidData, format!("Incorrect JSON Value: Expected Value::Object<Map<String, String>>, Got {:?}", json_value)))
                )
            })
        }
    }
}