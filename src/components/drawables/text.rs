use coffee::graphics::{Point, Color, HorizontalAlignment, VerticalAlignment, Window, Text as CoffeeText};
use specs::{Component, DenseVecStorage, World, Builder};
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Error, Result};
use serde::Deserialize;

pub const TEXT_LOAD_ID: &str = "text";

pub struct Text {
    pub content: Vec<String>,
    pub content_index: usize,
    pub position: Point,
    pub bounds: (f32, f32),
    pub size: f32,
    pub color: Color,
    pub h_align: HorizontalAlignment,
    pub v_align: VerticalAlignment,
    pub font: String,
}

impl Component for Text {
    type Storage = DenseVecStorage<Self>;
}

impl<'a> From<&'a Text> for CoffeeText<'a> {
    fn from(text: &'a Text) -> Self {
        Self {
            content: text.content.get(text.content_index).expect(format!("ERROR: Failed to get content string at index: {}", text.content_index).as_str()),
            position: text.position.clone(),
            bounds: text.bounds,
            size: text.size,
            color: text.color,
            horizontal_alignment: text.h_align,
            vertical_alignment: text.v_align
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct TextJSON {
    pub content: Vec<String>,
    pub content_index: usize,
    pub position_x: f32,
    pub position_y: f32,
    pub bounds_x: f32,
    pub bounds_y: f32,
    pub size: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub h_align: String,
    pub v_align: String,
    pub font: String,
}

impl From<&TextJSON> for Text {
    fn from(json: &TextJSON) -> Self {
        let vertical_alignment = match json.v_align.as_str() {
            "Top" => VerticalAlignment::Top,
            "Center" => VerticalAlignment::Center,
            "Bottom" => VerticalAlignment::Bottom,
            _ => panic!(format!("ERROR: json.v_align value: {:?} did not match any VerticalAlignment values", json.v_align))
        };

        let horizontal_alignment = match json.h_align.as_str() {
            "Left" => HorizontalAlignment::Left,
            "Center" => HorizontalAlignment::Center,
            "Right" => HorizontalAlignment::Right,
            _ => panic!(format!("ERROR: json.h_align value: {:?} did not match any HorizontalAlignment values", json.h_align))
        };

        Text {
            content: json.content.clone(),
            content_index: json.content_index,
            position: Point::from([json.position_x, json.position_y]),
            bounds: (json.bounds_x, json.bounds_y),
            size: json.size,
            color: Color::new(
                json.r,
                json.g,
                json.b,
                json.a
            ),
            h_align: horizontal_alignment,
            v_align: vertical_alignment,
            font: json.font.clone()
        }
    }
}

#[derive(Debug)]
pub struct TextLoader {
    text_json: TextJSON
}

impl ComponentLoader for TextLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let text_json = load_deserializable_from_json(json, TEXT_LOAD_ID)
            .map_err(|e| {
                Error::new(e)
            })?;

        Ok(Self{text_json})
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>> {
        Ok(builder.with(Text::from(&self.text_json)))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        self.text_json = load_deserializable_from_json(new_value, TEXT_LOAD_ID)?;
        Ok(())
    }

    fn get_component_name(&self) -> String {
        return TEXT_LOAD_ID.to_string()
    }
}