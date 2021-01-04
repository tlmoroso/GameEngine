#[cfg(feature="trace")]
use tracing::{instrument, trace, error};

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, RwLock};

use kira::sound::SoundId;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::AudioError;
use kira::playable::PlayableSettings;

use coffee::load::Task;

use serde::Deserialize;

use thiserror::Error;

use crate::load::{load_deserializable_from_file, LoadError};
use crate::globals::audio_controller::AudioControllerError::{FileLoadError, ManagerError, LoadSoundError};

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
            audio_manager: Arc::new(RwLock::new(AudioManager::new(AudioManagerSettings::default())
                .expect("Failed to create default AudioManager with default settings")))
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