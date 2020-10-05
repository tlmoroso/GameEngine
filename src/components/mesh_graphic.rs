use specs::{Component, World};
use specs::storage::DenseVecStorage;

use coffee::graphics::{Mesh, Window, Shape, Rectangle, Color};
use crate::load::{Loadable, ComponentLoadable};
use coffee::load::Task;
use serde_json::{Value, from_value};

use serde::Deserialize;
use crate::components::ComponentType;
use std::sync::{RwLock, Arc};

pub const MESH_GRAPHIC_FILE_ID: &str = "mesh_graphic";

#[derive(Deserialize, Debug)]
struct MeshGraphicJSON {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub stroke_width: f32,
}

#[derive(Debug)]
pub struct MeshGraphic {
    pub mesh: Mesh
}

impl Component for MeshGraphic {
    type Storage = DenseVecStorage<Self>;
}

impl Loadable for MeshGraphic {}
impl ComponentLoadable for MeshGraphic {}

impl MeshGraphic {
    pub fn load(_ecs: Arc<RwLock<World>>, _window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<ComponentType> {
        Task::new(|| {
            let mesh_json: MeshGraphicJSON = from_value(json_value)
                .expect("ERROR: could not translate Value to MeshGraphicJSON struct");
            let mut mesh = Mesh::new();
            mesh.stroke(
                Shape::Rectangle(Rectangle {
                    x: mesh_json.x,
                    y: mesh_json.y,
                    width: mesh_json.width,
                    height: mesh_json.height,
                }),
                Color::new(
                    mesh_json.r,
                    mesh_json.g,
                    mesh_json.b,
                    mesh_json.a
                ),
                mesh_json.stroke_width
            );

            Ok(ComponentType::MeshGraphic(
                MeshGraphic {
                    mesh
                }
            ))
        })
    }
}

