use crate::input::CustomInput;
use crate::load::{Loadable, load_json};

use coffee::graphics::{Window, Frame};
use coffee::{Timer};
use coffee::load::Task;

use specs::World;

use serde::Deserialize;
use serde_json::{Value, from_value};
use crate::scenes::test_scene::{TestScene, TEST_SCENE_FILE_ID};
use std::sync::{Arc, RwLock};

pub mod test_scene;

pub const SCENE_STACK_FILE_ID: &str = "scene_stack";
pub const SCENES_DIR: &str = "scenes";

pub trait Scene: Sync + Send {
    // Instance Methods
    fn update(&mut self, ecs: Arc<RwLock<World>>) -> SceneTransition;
    fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer);
    fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut CustomInput, window: Arc<RwLock<&mut Window>>);
}

#[derive(Deserialize, Debug)]
pub struct EntityVecJSON(Vec<String>);

#[derive(Deserialize, Debug)]
struct SceneVecJSON(Vec<String>);

pub enum SceneTransition {
    POP(u8),
    PUSH(Box<dyn Scene>),
    SWAP(Box<dyn Scene>),
    CLEAR,
    NONE,
}

pub struct SceneStack {
    pub stack: Vec<Box<dyn Scene>>,
    pub loaded: bool,
}
impl Loadable for SceneStack {}

impl SceneStack {
    pub fn load(ecs: Arc<RwLock<World>>, window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<Self> {
        let scene_vec: SceneVecJSON = from_value(json_value.clone())
            .expect(format!("ERROR: Could not translate json value into scene_vec: {}", json_value.clone()).as_str());

        let mut scene_stack: Vec<Box<dyn Scene>> = Vec::new();
        for scene_path in scene_vec.0 {
            let json_value = load_json(scene_path);
            let scene = match json_value.loadable_type.as_str() {
                TEST_SCENE_FILE_ID => TestScene::load(ecs.clone(), window.clone(), json_value.other_value),
                _ => panic!(format!("ERROR: scene loadable type invalid: {}", json_value.loadable_type)),
            };

            scene_stack.push(Box::new(scene
                .run(window.write().expect("ERROR: RwLock poisoned for window in SceneStack::load").gpu())
                .expect("ERROR: failed to run scene task")
            ));
        }

        Task::new(|| {
            Ok(
                SceneStack {
                stack: scene_stack,
                loaded: true
                }
            )
        })

    }

    pub fn update(&mut self, ecs: Arc<RwLock<World>>) {
        let transition: SceneTransition;

        if let Some(scene) = self.stack.last_mut() {
            transition = scene.update(ecs);
        } else {
            println!("ERROR: scene stack is empty");
            panic!();
        }

        match transition {
            SceneTransition::POP(_quantity) => {},
            SceneTransition::PUSH(_new_scene) => {},
            SceneTransition::SWAP(_new_scene) => {},
            SceneTransition::CLEAR => {},
            _ => {}
        }
    }

    pub fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer) {
        if let Some(scene) = self.stack.last_mut() {
            scene.draw(ecs, frame, timer);
        } else {
            println!("ERROR: scene stack is empty");
        }
    }

    pub fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut CustomInput, window: Arc<RwLock<&mut Window>>) {
        if let Some(scene) = self.stack.last_mut() {
            scene.interact(ecs, input, window);
        } else {
            println!("ERROR: scene stack is empty");
        } 
    }
}
