use specs::{Component, World};
use specs::storage::DenseVecStorage;

use coffee::graphics::{Point, Color, HorizontalAlignment, VerticalAlignment, Window};
use coffee::load::Task;

use crate::load::{Loadable, ComponentLoadable};
use crate::globals::{FontDict};

use serde::Deserialize;
use serde_json::{from_value, Value};
use crate::components::ComponentType;
use std::sync::{Arc, RwLock};

pub const TEXT_DISPLAY_FILE_ID: &str = "text_display";

const H_ALIGN: HorizontalAlignment = HorizontalAlignment::Center;
const V_ALIGN: VerticalAlignment = VerticalAlignment::Center;

#[derive(Deserialize, Debug)]
struct TextDisplayJSON {
    pub content: Vec<String>,
    pub position_x: f32,
    pub position_y: f32,
    pub bounds_x: f32,
    pub bounds_y: f32,
    pub size: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub font: String,
}

#[derive(Debug)]
pub struct TextDisplay {
    pub content: Vec<String>,
    pub position: Point,
    pub bounds: (f32, f32),
    pub size: f32,
    pub color: Color,
    pub h_align: HorizontalAlignment,
    pub v_align: VerticalAlignment,
    pub font: String,
}

impl Component for TextDisplay {
    type Storage = DenseVecStorage<Self>;
}

impl Loadable for TextDisplay {}
impl ComponentLoadable for TextDisplay {}

impl TextDisplay {
    pub fn load(ecs: Arc<RwLock<World>>, _window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<ComponentType> {
        let text_display_json: TextDisplayJSON = from_value(json_value)
        .expect("ERROR: failed to translate Value to TextDisplayJSON");
        let world = ecs
        .read()
        .expect("ERROR: RwLock for ecs poisoned in TextDisplay::load");

        let font_dict = world
            .fetch::<FontDict>();

        if !font_dict.0.read().expect("ERROR: RwLock poisoned for font dict in TextDisplay::load").contains_key(text_display_json.font.as_str()) {
            panic!(format!("ERROR: font name does not match any fonts: {}", text_display_json.font));
        }
        println!("TextDisplay::load complete");
        Task::new(|| {
            Ok(ComponentType::TextDisplay(
                TextDisplay {
                    content: text_display_json.content,
                    position: Point::from([text_display_json.position_x, text_display_json.position_y]),
                    bounds: (text_display_json.bounds_x, text_display_json.bounds_y),
                    size: text_display_json.size,
                    color: Color {
                        r: text_display_json.r,
                        g: text_display_json.g,
                        b: text_display_json.b,
                        a: text_display_json.a
                    },
                    h_align: H_ALIGN,
                    v_align: V_ALIGN,
                    font: text_display_json.font,
                }
            ))
        })
        
    }
}