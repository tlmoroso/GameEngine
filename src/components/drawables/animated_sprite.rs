use coffee::graphics::{Image, Window, Sprite as CoffeeSprite, Rectangle, Point};
use specs::{Component, VecStorage, World, Builder};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Error, Result};
use crate::load::LoadError::LoadIDError;
use crate::globals::image_dict::ImageDict;
use serde::Deserialize;

pub const ANIMATED_SPRITE_LOAD_ID: &str = "animated_sprite";

pub struct AnimatedSprite {
    pub sprite: CoffeeSprite,
    pub start_frame: u16,
    pub end_frame: u16,
    pub frame_pause: u16,
    pub frame_pause_counter: u16,
    pub image: String
}

impl Component for AnimatedSprite {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AnimatedSpriteJSON {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub position: [f32; 2],
    pub scale: (f32, f32),
    pub start_frame: u16,
    pub end_frame: u16,
    pub frame_pause: u16,
    pub image: String
}

#[derive(Debug)]
pub struct AnimatedSpriteLoader {
    pub(crate) sprite_json: AnimatedSpriteJSON
}

impl AnimatedSpriteLoader {
    pub fn build_sprite(&self, ecs: &World) -> Result<AnimatedSprite> {
        let sprite = AnimatedSprite {
            sprite: CoffeeSprite {
                source: Rectangle {
                    x: self.sprite_json.x,
                    y: self.sprite_json.y,
                    width: self.sprite_json.width,
                    height: self.sprite_json.height
                },
                position: Point::from(self.sprite_json.position),
                scale: self.sprite_json.scale
            },
            start_frame: self.sprite_json.start_frame,
            end_frame: self.sprite_json.end_frame,
            frame_pause: self.sprite_json.frame_pause,
            frame_pause_counter: 0,
            image: self.sprite_json.image.clone()
        };

        Ok(sprite)
    }
}

impl ComponentLoader for AnimatedSpriteLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let sprite_json = load_deserializable_from_json(json, ANIMATED_SPRITE_LOAD_ID)
            .map_err(|e| {
                Error::new(e)
            })?;

        Ok(Self{sprite_json})
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>> {
        let sprite = self.build_sprite(ecs)?;
        Ok(builder.with(sprite))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        self.sprite_json = load_deserializable_from_json(new_value, ANIMATED_SPRITE_LOAD_ID)?;
        Ok(())
    }

    fn get_component_name(&self) -> String {
        return ANIMATED_SPRITE_LOAD_ID.to_string()
    }
}