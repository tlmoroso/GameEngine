use specs::{Component, DenseVecStorage, World, Builder};
use crate::components::ComponentLoader;
use coffee::graphics::Window;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Result, Error};
use kira::instance::{InstanceId, InstanceSettings, StopInstanceSettings, PauseInstanceSettings, ResumeInstanceSettings};
use serde::Deserialize;

pub const DEFAULT_SOUND_LOAD_ID: &str = "default_sound";

#[derive(Deserialize, Debug, Clone)]
pub struct DefaultSound {
    pub sound_name: String,
    #[serde(skip)]
    pub instance_id: Option<InstanceId>,
    pub play_flag: bool,
    // pub play_settings: InstanceSettings,
    // pub pause_settings: PauseInstanceSettings,
    // pub resume_settings: ResumeInstanceSettings,
    // pub stop_settings: StopInstanceSettings,
}

impl Component for DefaultSound {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug)]
pub struct DefaultSoundLoader {
    pub(crate) sound_component: DefaultSound
}

impl ComponentLoader for DefaultSoundLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let component = load_deserializable_from_json(json, DEFAULT_SOUND_LOAD_ID)
            .map_err(|e| {
                Error::new(e)
            })?;

        Ok(Self{sound_component: component})
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>> {
        Ok(builder.with(self.sound_component.clone()))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<(), Error> {
        self.sound_component = load_deserializable_from_json(new_value, DEFAULT_SOUND_LOAD_ID)
            .map_err(|e| {Error::new(e)})?;

        Ok(())
    }

    fn get_component_name(&self) -> String {
        return DEFAULT_SOUND_LOAD_ID.to_string()
    }
}