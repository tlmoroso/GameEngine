use serde::Deserialize;
use specs::{Component, VecStorage, World, Builder};
use glam::{Vec2, Mat4, Quat};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use std::sync::{Arc, Mutex, RwLock};
use luminance_glfw::GL33Context;
use anyhow::Error;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            scale: Vec2::ZERO,
            rotation: 0.0
        }
    }
}

impl Component for Transform { type Storage = VecStorage<Self>; }

impl Transform {
    pub fn to_model(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(0.0),
            Quat::from_rotation_z(self.rotation),
            self.translation.extend(0.0)
        )
    }
}

#[derive(Debug)]
pub struct TransformLoader {
    json: TransformJSON
}

pub const TRANSFORM_LOAD_ID: &str = "transform";

#[derive(Deserialize, Debug, Clone)]
pub struct TransformJSON {
    translation: [f32; 2],
    scale: [f32;2],
    rotation: f32
}

impl ComponentLoader for TransformLoader {
    fn from_json(json: JSONLoad) -> anyhow::Result<Self> where Self: Sized {
        let json = load_deserializable_from_json(&json, &TRANSFORM_LOAD_ID)?;

        Ok(Self {
            json
        })
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: Arc<RwLock<World>>, context: Option<Arc<RwLock<GL33Context>>>) -> anyhow::Result<LazyBuilder<'a>> {
        Ok(builder.with(
            Transform {
                translation: Vec2::from(self.json.translation),
                scale: Vec2::from(self.json.scale),
                rotation: self.json.rotation
            }
        ))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> anyhow::Result<()> {
        if new_value.load_type_id == TRANSFORM_LOAD_ID {
            self.json = load_deserializable_from_json(&new_value, &TRANSFORM_LOAD_ID)?;
            Ok(())
        } else {
            Err(
                Error::msg("New Value has incorrect load ID")
            )
        }
    }

    fn get_component_name(&self) -> String {
        TRANSFORM_LOAD_ID.to_string()
    }
}