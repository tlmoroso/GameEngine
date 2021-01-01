use coffee::graphics::{Mesh, Target, Color, Window, Rectangle, Shape, Point};
use crate::components::drawables::Drawable;
use specs::{VecStorage, World, Builder, Component};
use serde::Deserialize;
use crate::components::ComponentLoader;
use crate::load::{JSONLoad, load_deserializable_from_json};
use specs::world::LazyBuilder;
use anyhow::{Error, Result};
use serde_json::from_value;
use crate::load::LoadError::LoadIDError;

pub const SHAPES_LOAD_ID: &str = "shapes";

pub struct Shapes {
    pub mesh: Mesh
}

impl Component for Shapes {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub(crate) enum PaintType {
    Fill,
    Stroke {
        width: f32
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub(crate) struct Description {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    paint_type: PaintType
}

impl Description {
    pub fn get_color(&self) -> Color {
        Color::new(self.r, self.g, self.b, self.a)
    }

    pub fn get_stroke_width(&self) -> Option<f32> {
        return if let PaintType::Stroke { width} = self.paint_type {
            Some(width)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) enum ShapeJSON {
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Circle {
        center: [f32; 2],
        radius: f32
    },
    Ellipse {
        center: [f32; 2],
        horizontal_radius: f32,
        vertical_radius: f32,
        rotation: f32
    },
    Polyline {
        points: Vec<[f32; 2]>
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MeshJSON {
    pub shape: ShapeJSON,
    pub description: Description
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ShapesJSON {
    pub shapes: Vec<MeshJSON>
}

impl From<&ShapesJSON> for Shapes {
    fn from(json: &ShapesJSON) -> Self {
        let mut mesh = Mesh::new();

        for mesh_json in &json.shapes {
            let shape = match mesh_json.shape.clone() {
                ShapeJSON::Rectangle {x, y, width, height} => {
                    Shape::Rectangle(
                        Rectangle {
                            x,
                            y,
                            width,
                            height
                        }
                    )
                }
                ShapeJSON::Circle { center, radius } => {
                    Shape::Circle {
                        center: Point::from(center),
                        radius
                    }
                }
                ShapeJSON::Ellipse {center, horizontal_radius, vertical_radius, rotation} => {
                    Shape::Ellipse {
                        center: Point::from(center),
                        horizontal_radius,
                        vertical_radius,
                        rotation
                    }
                }
                ShapeJSON::Polyline { points } => {
                    Shape::Polyline {
                        points: points.iter().map(|point| {
                            Point::from(point.clone())
                        }).collect()
                    }
                }
            };

            if let PaintType::Stroke { width } = mesh_json.description.paint_type {
                mesh.stroke(shape, mesh_json.description.get_color(), width);
            } else {
                mesh.fill(shape, mesh_json.description.get_color())
            }
        }

        return Shapes{mesh}
    }
}

#[derive(Debug)]
pub struct ShapesLoader {
    shapes_json: ShapesJSON
}

impl ComponentLoader for ShapesLoader {
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let shapes_json = load_deserializable_from_json(json, SHAPES_LOAD_ID)
            .map_err(|e| {
                Error::new(e)
            })?;

        Ok(Self {
            shapes_json
        })
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: &World, window: &Window) -> Result<LazyBuilder<'a>, Error> {
        Ok(builder.with(Shapes::from(&self.shapes_json)))
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        self.shapes_json = load_deserializable_from_json(new_value, SHAPES_LOAD_ID)?;
        Ok(())
    }

    fn get_component_name(&self) -> String {
        return SHAPES_LOAD_ID.to_string()
    }
}