use crate::globals::FontDictError::{FontDictFileLoadError, FontDictJSONLoadError, FontDictFileReadError, FontDictFontSizeError};
use crate::load::{load_json, LoadError, build_task_error, load_deserializable_from_file};

use std::collections::{HashMap};
use std::fs;
use std::sync::{Arc, RwLock};
use std::io::ErrorKind;

use coffee::graphics::{Font, Window, Image};
use coffee::load::{Task, Join};

use specs::{World};

use serde_json::{Value, from_value};
use serde::Deserialize;

use thiserror::Error;
use anyhow::Error;

#[cfg(feature="trace")]
use tracing::{instrument, trace, error};
use crate::globals::ImageDictError::ImageDictFileLoadError;
use kira::sound::SoundId;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::playable::PlayableSettings;
use crate::globals::AudioControllerError::{FileLoadError, ManagerError, LoadSoundError};
use kira::AudioError;

pub const FONT_DICT_LOAD_ID: &str = "font_dict";

#[derive(Default)]
pub struct FontDict(pub HashMap<String, Font>);

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
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: FontDictLoader::new");
        let new = Self {
            path: file_path
        };
        #[cfg(feature="trace")]
        trace!("EXIT: FontDictLoader::new");
        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _ecs, _window)))]
    pub fn load(self) -> Task<FontDict> {
        #[cfg(feature="trace")]
        trace!("ENTER: FontDictLoader::load");
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
        #[cfg(feature="trace")]
        trace!("Value: {:#?} successfully loaded from: {:#?}", json_value, self.path);

        if json_value.load_type_id != FONT_DICT_LOAD_ID {
            return build_task_error(
                LoadError::LoadIDError {
                    actual: json_value.load_type_id,
                    expected: FONT_DICT_LOAD_ID.to_string(),
                },
                ErrorKind::InvalidData
            )
        }
        #[cfg(feature="trace")]
        trace!("Value type ID: {} correctly matches FONT_DICT_FILE_ID", json_value.load_type_id.clone());

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
        #[cfg(feature="trace")]
        trace!("Value: {} successfully transformed into FontDictLoaderJSON", json_value.actual_value.clone());

        for (index, (font_name, font_path)) in fonts.fonts.into_iter().enumerate() {
            let font = map_err_return!(
                fs::read(font_path.clone()),
                |e| { build_task_error(
                    FontDictFileReadError {
                        path: font_path.clone(),
                        source: e
                    },
                    ErrorKind::InvalidData
                )}
            );
            #[cfg(feature="trace")]
            trace!("Font: {} successfully loaded from: {}", font_name.clone(), font_path);

            if font.len() <= FONT_FILE_SIZE {
                unsafe {
                    for (i, byte) in font.iter().enumerate() {
                        FONT_BYTES[index][i] = *byte
                    }
                };
            } else {
                let error = FontDictFontSizeError {
                    font_size: font.len(),
                    font_name: font_name.clone(),
                    font_path: font_path.clone()
                };
                #[cfg(feature="trace")]
                error!("ERROR: Could not load font: {:#?}", error);

                return build_task_error(
                    error,
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

        let task = font_task.map(|font_dict| {
            FontDict(font_dict)
        });
        #[cfg(feature="trace")]
        trace!("EXIT: FontDictLoader::load");
        return task
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



pub const IMAGE_DICT_LOAD_ID: &str = "image_dict";

#[derive(Default, Debug)]
pub struct ImageDict(pub HashMap<String, Image>);

pub const IMAGES_DIR: &str = "images/";

#[derive(Deserialize, Debug)]
pub struct ImageDictLoader {
    path: String
}

#[derive(Deserialize, Debug, Clone)]
struct ImageDictJSON {
    images: HashMap<String, String>
}

impl ImageDictLoader {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: ImageDictLoader::new");
        let new = Self {
            path: file_path
        };
        #[cfg(feature="trace")]
        trace!("EXIT: ImageDictLoader::new");
        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _ecs, _window)))]
    pub fn load(self) -> Task<ImageDict> {
        #[cfg(feature="trace")]
        trace!("ENTER: ImageDictLoader::load");
        let mut image_task = Task::new(|| { Ok(
            HashMap::new()
        )});

        let image_dict_json: ImageDictJSON = map_err_return!(
            load_deserializable_from_file(self.path.as_str(), IMAGE_DICT_LOAD_ID),
            |e| { build_task_error(
                ImageDictFileLoadError {
                    path: self.path,
                    var_name: stringify!(self.path).to_string(),
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );

        #[cfg(feature="trace")]
        trace!("ImageDictJSON: {:#?} successfully loaded from: {:#?}", image_dict_json, self.path);

        for (image_name, image_path) in image_dict_json.images {
            #[cfg(feature="trace")]
            trace!("Adding {:#?} at {:#?} to ImageDict", image_name.clone(), image_path.clone());

            image_task = (
                Image::load(image_path),
                image_task
            )
                .join()
                .map(|(image, mut image_dict)| {
                    image_dict.insert(image_name, image);
                    return image_dict
                })
        }

        let task = image_task.map(|image_dict| {
            ImageDict(image_dict)
        });

        #[cfg(feature="trace")]
        trace!("EXIT: ImageDictLoader::load");
        return task
    }
}

#[derive(Error, Debug)]
pub enum ImageDictError {
    #[error("Error loading JSON Value for ImageDictLoader from: {var_name} = {path}")]
    ImageDictFileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    }
}

pub const AUDIO_CONTROLLER_LOAD_ID: &str = "audio_controller";
pub const AUDIO_DIR: &str = "audio/";

#[derive(Default, Debug)]
pub struct AudioDict(pub HashMap<String, SoundId>);

pub struct AudioController {
    pub audio_lib: AudioDict,
    pub audio_manager: Arc<RwLock<AudioManager>>
}

unsafe impl Send for AudioController {}
unsafe impl Sync for AudioController {}

impl Default for AudioController {
    fn default() -> Self {
        return AudioController {
            audio_lib: AudioDict(HashMap::new()),
            audio_manager: Arc::new(RwLock::new(AudioManager::new(AudioManagerSettings::default()).expect("Failed to create default AudioManager with default settings")))
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AudioControllerLoader {
    path: String
}

#[derive(Deserialize, Debug, Clone)]
struct AudioControllerJSON {
    sounds: HashMap<String, String>,
}

impl AudioControllerLoader {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: AudioControllerLoader::new");
        let new = Self {
            path: file_path
        };
        #[cfg(feature="trace")]
        trace!("EXIT: AudioControllerLoader::new");
        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _ecs, _window)))]
    pub fn load(self, settings: AudioManagerSettings) -> Task<AudioController> {
        #[cfg(feature="trace")]
        trace!("ENTER: AudioControllerLoader::load");
        Task::new(|| {
            let audio_controller_json: AudioControllerJSON =
            load_deserializable_from_file(self.path.as_str(), AUDIO_CONTROLLER_LOAD_ID)
                .map_err(|e| {
                    let error: coffee::Error = FileLoadError {
                        path: self.path,
                        var_name: stringify!(self.path).to_string(),
                        source: e
                    }.into();

                    return error
                })?;

            #[cfg(feature="trace")]
            trace!("AudioControllerJSON: {:#?} successfully loaded from: {:#?}", audio_controller_json, self.path);

            let mut audio_manager = AudioManager::new(settings.clone())
                .map_err(|e| {
                    let error: coffee::Error = ManagerError {
                        settings,
                        source: e
                    }.into();

                    return error
                })?;
            let mut audio_dict = HashMap::new();

            for (audio_name, audio_path) in audio_controller_json.sounds {
                #[cfg(feature="trace")]
                trace!("Adding {:#?} at {:#?} to AudioDict", audio_name.clone(), audio_path.clone());
                let audio = audio_manager.load_sound(audio_path.clone(), PlayableSettings::new())
                    .map_err(|e| {
                        let error: coffee::Error = LoadSoundError {
                            sound_name: audio_name.clone(),
                            sound_path: audio_path,
                            settings: PlayableSettings::new()
                        }.into();

                        return error
                    })?;

                audio_dict.insert(audio_name, audio);
            }

            #[cfg(feature="trace")]
            trace!("EXIT: AudioControllerLoader::load");
            return Ok(AudioController {
                audio_lib: AudioDict(audio_dict),
                audio_manager: Arc::new(RwLock::new(audio_manager))
            })
        })
    }
}

#[derive(Error, Debug)]
pub enum AudioControllerError {
    #[error("Error loading JSON Value for AudioControllerLoader from: {var_name} = {path}")]
    FileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    },
    #[error("Error creating AudioManager with settings: {settings:#?}")]
    ManagerError {
        settings: AudioManagerSettings,
        source: AudioError
    },
    #[error("Error loading sound: {sound_name} from {sound_path} in AudioManager with settings: {settings:#?}")]
    LoadSoundError {
        sound_name: String,
        sound_path: String,
        settings: PlayableSettings
    }
}

impl Into<coffee::Error> for AudioControllerError {
    fn into(self) -> coffee::Error {
        coffee::Error::IO(
            std::io::Error::new(
                ErrorKind::InvalidData,
                format!("{:#?}", self)
            )
        )
    }
}