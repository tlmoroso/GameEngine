use coffee::graphics::{Image, Window, Sprite as CoffeeSprite, Rectangle, Point};
use specs::{Component, VecStorage, World, Builder};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Error, Result};
use crate::load::LoadError::LoadIDError;
use crate::globals::image_dict::ImageDict;
use serde::Deserialize;

pub const SPRITE_LOAD_ID: &str = "texture";

pub struct Sprite {
    pub sprite: Sprite,
    pub image: String
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SpriteJSON {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub position: [f32; 2],
    pub scale: (f32, f32),
    pub image: String
}

#[derive(Debug)]
pub struct SpriteLoader {
    pub(crate) sprite_json: SpriteJSON
}

impl SpriteLoader {
    pub fn build_sprite(&self, ecs: &World) -> Result<Sprite> {
        // let image_dict = ecs.fetch::<ImageDict>();
        // let image = image_dict.0.get(self.sprite_json.image.as_str())
        //     .expect(format!("ERROR: image name: {:#?} did not match any values in image_dict: {:#?}", self.sprite_json.image, image_dict.0).as_str()).clone();

        let sprite = Sprite {
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
            image: self.sprite_json.image.clone()
        };

        Ok(sprite)
    }
}

impl ComponentLoader for SpriteLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let sprite_json = load_deserializable_from_json(json, SPRITE_LOAD_ID)
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
        self.sprite_json = load_deserializable_from_json(new_value, SPRITE_LOAD_ID)?;
        Ok(())
    }

    fn get_component_name(&self) -> String {
        return SPRITE_LOAD_ID.to_string()
    }
}