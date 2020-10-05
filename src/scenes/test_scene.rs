use crate::scenes::{SceneTransition, Scene, EntityVecJSON};
use crate::input::CustomInput;
use crate::systems::move_player::MovePlayer;
use crate::components::{text_display::TextDisplay, mesh_graphic::MeshGraphic};
use crate::load::{Loadable, SceneLoadable, load_json};
use crate::globals::FontDict;

use coffee::graphics::{Window, Frame, Text};
use coffee::{Timer};
use coffee::load::{Task};

use specs::{World, WorldExt, RunNow, Join};

use serde_json::{Value, from_value};
use crate::entities::LoadableEntity;
use std::sync::{Arc, RwLock};
use std::ops::Deref;

pub const TEST_SCENE_FILE_ID: &str = "test_scene";

pub struct TestScene {
    pub text: &'static str,
    pub frame_counter: u32,
}

impl Loadable for TestScene {}
impl SceneLoadable for TestScene {}

impl TestScene {
    pub fn load(ecs: Arc<RwLock<World>>, window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<Self> {
        let entity_vec: EntityVecJSON = from_value(json_value)
            .expect("ERROR: could not translate json to entity_vec in TestScene::load");
        for entity_path in entity_vec.0 {
            let json_value = load_json(entity_path);
            let entity_task = LoadableEntity::load(ecs.clone(), window.clone(), json_value.other_value);
            entity_task.run(window.write().expect("ERROR: RwLock poisoned for window in TestScene::load").gpu());
        }
        Task::new(||{
            Ok(
                TestScene {
                    text: "TestScene",
                    frame_counter: 0,
                }
            )
        })
    }
}


impl Scene for TestScene {
    fn update(&mut self, _ecs: Arc<RwLock<World>>) -> SceneTransition {
        self.frame_counter += 1;
        return SceneTransition::NONE;
    }

    fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, _timer: &Timer) {
        let world_mut = ecs
            .write()
            .expect("ERROR: RwLock poisoned in TestScene::draw");

        let txt_store = world_mut.read_storage::<TextDisplay>();
        let mesh_store = world_mut.read_storage::<MeshGraphic>();

        for (txt, mesh) in (&txt_store, &mesh_store).join() {
            mesh.mesh.draw(&mut frame.as_target());
            let mut fetch_font_dict = world_mut
                .fetch_mut::<FontDict>();

            let mut font_dict = fetch_font_dict.0
                .write()
                .expect("ERROR: font dict was poisoned during TestScene::draw");

            let mut font = font_dict.get_mut(&txt.font).unwrap();

            font.add(Text {
                content: &txt.content[0],
                position: txt.position.clone(),
                bounds: txt.bounds,
                size: txt.size,
                color: txt.color,
                horizontal_alignment: txt.h_align,
                vertical_alignment: txt.v_align,
            });

            font.draw(&mut frame.as_target());
        }
    }

    fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut CustomInput, _window: Arc<RwLock<&mut Window>>) {
        for key in input.get_keys_pressed() {
            println!("{:?} PRESSED", key);
        }

        for key in input.get_keys_released() {
            println!("{:?} RELEASED", key);
        }

        let mut move_player = MovePlayer {
            keys_pressed: input.get_keys_pressed().clone()
        };
        move_player.run_now(ecs.read().expect("ERROR: RwLock poisoned in TestScene::interact").deref());
    }
}
