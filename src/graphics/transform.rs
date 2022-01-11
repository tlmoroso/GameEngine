use serde::Deserialize;
use specs::{Component, VecStorage, World, Builder};
use glam::{Vec2, Mat4, Quat};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json, LoadError};
use specs::world::LazyBuilder;
use std::sync::{Arc, Mutex, RwLock};
use luminance_glfw::GL33Context;
use anyhow::Error;
use thiserror::Error;
use atomic_float::AtomicF32;

#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};
use crate::graphics::transform::TransformLoaderError::{DeserializeError, LoadTypeIDError};
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug)]
pub struct Transform {
    pub translation: [AtomicF32; 2],
    pub scale: [AtomicF32; 2],
    pub rotation: AtomicF32
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [AtomicF32::new(0.0), AtomicF32::new(0.0)],
            scale: [AtomicF32::new(0.0), AtomicF32::new(0.0)],
            rotation: AtomicF32::new(0.0)
        }
    }
}

impl Component for Transform { type Storage = VecStorage<Self>; }

impl Transform {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn to_model(&self) -> Mat4 {
        let scale = Vec2::new(self.scale[0].load(Relaxed), self.scale[1].load(Relaxed))
            .extend(0.0);

        let translation = Vec2::new(self.translation[0].load(Relaxed), self.translation[1].load(Relaxed))
            .extend(0.0);

        let model = Mat4::from_scale_rotation_translation(
            scale,
            Quat::from_rotation_z(self.rotation.load(Relaxed)),
            translation
        );
        #[cfg(feature = "trace")]
        debug!("Created model matrix for entity from transform component: {:?}", model);

        return model
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
    #[cfg_attr(feature = "trace", instrument)]
    fn from_json(json: JSONLoad) -> anyhow::Result<Self> where Self: Sized {
        let transform_json: TransformJSON = load_deserializable_from_json(&json, &TRANSFORM_LOAD_ID)
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to convert JSONLoad object: ({:?}) into TransformJSON value", json.clone());

                DeserializeError {
                    source: e,
                    json: json.clone()
                }
            })?;
        #[cfg(feature = "trace")]
        debug!("Successfully converted JSONLoad object: ({:?}) into TransformJSON value: {:?}", json.clone(), transform_json.clone());

        Ok(Self {json: transform_json})
    }

    #[cfg_attr(feature = "trace", instrument(skip(builder, _ecs)))]
    fn load_component<'a>(&self, builder: LazyBuilder<'a>, _ecs: Arc<RwLock<World>>) -> anyhow::Result<LazyBuilder<'a>> {
        let transform = Transform {
            translation: [AtomicF32::new(self.json.translation[0]), AtomicF32::new(self.json.translation[1])],
            scale: [AtomicF32::new(self.json.scale[0]), AtomicF32::new(self.json.scale[1])],
            rotation: AtomicF32::new(self.json.rotation)
        };

        #[cfg(feature = "trace")]
        debug!("Created new transform component: {:?}", transform);

        Ok(builder.with(
            transform
        ))
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn set_value(&mut self, new_value: JSONLoad) -> anyhow::Result<()> {
        if new_value.load_type_id == TRANSFORM_LOAD_ID {
            #[cfg(feature = "trace")]
            let old_value = self.json.clone();

            self.json = load_deserializable_from_json(&new_value, &TRANSFORM_LOAD_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to convert JSONLoad object: ({:?}) into TransformJSON value", new_value.clone());

                    DeserializeError {
                        source: e,
                        json: new_value.clone()
                    }
                })?;
            #[cfg(feature = "trace")]
            debug!("Successfully replaced current transform JSON value: ({:?}) with new value: {:?}", old_value, self.json.clone());

            Ok(())
        } else {
            #[cfg(feature = "trace")]
            error!("Given load-type ID: ({:?}) does not match expected ID: {:?}", new_value.load_type_id.clone(), TRANSFORM_LOAD_ID.to_string());

           Err(Error::new(LoadTypeIDError {
                actual: new_value.load_type_id,
                expected: TRANSFORM_LOAD_ID.to_string()
            }))
        }
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn get_component_name(&self) -> String {
        TRANSFORM_LOAD_ID.to_string()
    }
}

#[derive(Error, Debug)]
pub enum TransformLoaderError {
    #[error("Failed to convert JSONLoad object: ({json:?}) to TransformJSON value")]
    DeserializeError {
        source: LoadError,
        json: JSONLoad
    },

    #[error("Given load-type ID: ({actual}) does not match expected ID: {expected}")]
    LoadTypeIDError {
        actual: String,
        expected: String
    }
}
