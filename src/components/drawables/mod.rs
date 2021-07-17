pub mod shapes;
pub mod text;
pub mod animated_sprite;
pub mod sprite;

use coffee::graphics::{Target, Window};
use serde::Deserialize;
use specs::{Component, VecStorage, World, Builder};
use crate::components::drawables::shapes::{Shapes, ShapesJSON};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Error, Result};
use crate::components::drawables::text::{Text, TextJSON};
use crate::components::drawables::sprite::{Sprite, SpriteJSON, SpriteLoader};
use crate::components::drawables::animated_sprite::{AnimatedSprite, AnimatedSpriteJSON, AnimatedSpriteLoader};

pub const DRAWABLE_LOAD_ID: &str = "drawable";

pub struct Drawable {
    pub shapes: Option<Vec<Shapes>>,
    pub text: Option<Vec<Text>>,
    pub sprites: Option<Vec<Sprite>>,
    pub animated_sprites: Option<Vec<AnimatedSprite>>
}

impl Component for Drawable {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Debug, Clone)]
struct DrawableJSON {
    #[serde(default)]
    pub shapes: Option<Vec<ShapesJSON>>,
    #[serde(default)]
    pub text: Option<Vec<TextJSON>>,
    #[serde(default)]
    pub sprites: Option<Vec<SpriteJSON>>,
    #[serde(default)]
    pub animated_sprites: Option<Vec<AnimatedSpriteJSON>>
}

#[derive(Debug)]
pub struct DrawableLoader {
    drawable_json: DrawableJSON
}

impl ComponentLoader for DrawableLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let drawable_json = load_deserializable_from_json(json, DRAWABLE_LOAD_ID)
            .map_err(|e| { Error::new(e) })?;

        Ok(Self{ drawable_json })
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>> {
        let meshes = self.drawable_json.shapes.as_deref().and_then(|shapes| {
            Some(shapes.iter().map(|shape| shape.into()).collect())
        });

        let text = self.drawable_json.text.as_deref().and_then(|text| {
            Some(text.iter().map(|text| text.into()).collect())
        });

        let sprites = self.drawable_json.sprites.as_deref().and_then(|sprites| {
            let mut sprite_loader: Option<SpriteLoader> = None;

            Some(sprites.iter().map(|sprite_json| {
                if let Some(loader) = &mut sprite_loader {
                    loader.sprite_json = sprite_json.clone();
                    loader.build_sprite(ecs).expect(format!("ERROR: failed to build texture from json: {:#?}", sprite_json).as_str())
                } else {
                    let loader = SpriteLoader {
                        sprite_json: sprite_json.clone()
                    };
                    let sprite = loader.build_sprite(ecs).expect(format!("ERROR: failed to build texture from json: {:#?}", sprite_json).as_str());
                    sprite_loader = Some(loader);
                    return sprite
                }
            }).collect())
        });

        let animated_sprites = self.drawable_json.animated_sprites.as_deref().and_then(|sprites| {
            let mut sprite_loader: Option<AnimatedSpriteLoader> = None;

            Some(sprites.iter().map(|sprite_json| {
                if let Some(loader) = &mut sprite_loader {
                    loader.sprite_json = sprite_json.clone();
                    loader.build_sprite(ecs).expect(format!("ERROR: failed to build animated texture from json: {:#?}", sprite_json).as_str())
                } else {
                    let loader = AnimatedSpriteLoader {
                        sprite_json: sprite_json.clone()
                    };
                    let sprite = loader.build_sprite(ecs).expect(format!("ERROR: failed to build animated texture from json: {:#?}", sprite_json).as_str());
                    sprite_loader = Some(loader);
                    return sprite
                }
            }).collect())
        });

        Ok(builder.with(Drawable{ shapes: meshes, text, sprites, animated_sprites }))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        let drawable_json: DrawableJSON = load_deserializable_from_json(new_value, DRAWABLE_LOAD_ID)?;
        self.drawable_json.shapes = drawable_json.shapes;
        self.drawable_json.text = drawable_json.text;
        self.drawable_json.sprites = drawable_json.sprites;
        self.drawable_json.animated_sprites = drawable_json.animated_sprites;

        Ok(())
    }

    fn get_component_name(&self) -> String {
        return DRAWABLE_LOAD_ID.to_string()
    }
}