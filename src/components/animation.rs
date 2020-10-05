use specs::{Component, DenseVecStorage, World};

use coffee::graphics::{Image, Sprite, Point, Rectangle, Window};
use coffee::load::Task;

use crate::components::position::Position;
use crate::globals::ImageDict;
use crate::load::{Loadable, ComponentLoadable};

use serde_json::Value;
use serde::Deserialize;
use crate::components::{ComponentType};
use std::sync::{RwLock, Arc};

pub const ANIMATION_FILE_ID: &str = "animation";

#[derive(Deserialize, Debug, Clone)]
struct AnimationJSON {
    pub image: String,
    pub current_frame: u16,
    pub start_frame: u16,
    pub end_frame: u16,
    pub total_frames: u16,
    pub dimensions_x: u16,
    pub dimensions_y: u16,
    pub scale_x: f32,
    pub scale_y: f32
}

#[derive(Debug)]
pub struct Animation {
    pub image: Image,
    pub current_frame: u16, // frames are 1-indexed
    pub start_frame: u16,
    pub end_frame: u16,
    pub total_frames: u16,
    pub dimensions: (u16, u16),
    pub scale: (f32, f32),
}

impl Component for Animation {
    type Storage = DenseVecStorage<Self>;
}

impl Loadable for Animation {}
impl ComponentLoadable for Animation {}

impl Animation {
    pub fn load(ecs: Arc<RwLock<World>>, window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<ComponentType> {
        let mut world = ecs
            .write()
            .expect("ERROR: RwLock poisoned in Animation::load");

        let image_dict = world
            .get_mut::<ImageDict>()
            .expect("ERROR: ImageDict does not exist in Animation load");

        let animation_json: AnimationJSON = serde_json::from_value(json_value)
            .expect("ERROR: could not translate value into AnimationJSON");

        let image = image_dict.0
            .get(animation_json.image.as_str())
            .map_or_else(
                || {Image::new(window.write().expect("ERROR: RwLock poisoned for window in Animation::load").gpu(), animation_json.image.clone())
                    .expect("ERROR: Image failed to load in Animation::load")},
                |image_ref| image_ref.clone()
            );

        image_dict.0.insert(animation_json.image.clone(), image.clone());
        // let image_clone = image.clone();

        let animation_json_clone = animation_json.clone();

        Task::new(move || {

            Ok(ComponentType::Animation(Animation {
                    image,//: image_clone,
                    current_frame: animation_json_clone.current_frame,
                    start_frame: animation_json_clone.start_frame,
                    end_frame: animation_json_clone.end_frame,
                    total_frames: animation_json_clone.total_frames,
                    dimensions: (animation_json_clone.dimensions_x, animation_json_clone.dimensions_y),
                    scale: (animation_json_clone.scale_x, animation_json_clone.scale_y)
                }
            ))
        })
    }
}



impl Animation {
    pub fn create_sprite(&mut self, pos: &Position) -> Sprite {
        let frame_width = self.dimensions.0/self.total_frames;
        let frame_height = self.dimensions.1;
        let frame_x = frame_width * (self.current_frame - 1);
        let frame_y = pos.y;


        self.current_frame += 1;

        Sprite {
            source: Rectangle {
                x: frame_x,
                y: frame_y,
                width: frame_width,
                height: frame_height,
            },
            position: Point::new(pos.x.into(), pos.y.into()),
            scale: self.scale,
        }
    }
}
