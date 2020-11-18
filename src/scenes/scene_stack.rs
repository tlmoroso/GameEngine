use crate::scenes::{Scene, SceneLoader};
use crate::load::{load_json, JSONLoad};

use specs::{World};

use coffee::graphics::{Window, Frame};
use coffee::load::{Task, Join};
use coffee::Timer;
use coffee::input::Input;

use std::io::ErrorKind;
use std::marker::PhantomData;

use serde_json::Value;

pub enum SceneTransition<T: Input> {
    POP(u8),
    PUSH(Box<dyn Scene<T>>),
    SWAP(Box<dyn Scene<T>>),
    CLEAR,
    NONE,
}

pub struct SceneStackLoader<T: 'static + Input> {
    scene_stack_file: String,
    scene_factory: fn(JSONLoad) -> Box<dyn SceneLoader<T>>
}

impl<T: 'static + Input> SceneStackLoader<T> {
    pub fn new(file_path: String, scene_factory: fn(JSONLoad) -> Box<dyn SceneLoader<T>>) -> Self {
        Self {
            scene_stack_file: file_path,
            scene_factory
        }
    }

    pub fn load(self, ecs: &mut World, window: &Window) -> Task<SceneStack<T>> {
        let json_value = load_json(&self.scene_stack_file).unwrap();
        return if let Value::Array(scene_paths) = json_value.actual_value {
            let mut scene_task = Task::new(|| {Ok(
                Vec::new()
            )});

            for scene_path in scene_paths {
                if let Value::String(scene_path) = scene_path {
                    let scene_value = load_json(&scene_path).unwrap();
                    let scene_loader = (self.scene_factory)(scene_value);
                    scene_task = (
                        scene_loader.load_scene(ecs, window),
                        scene_task
                    )
                        .join()
                        .map(|(scene, mut scene_vec)| {
                            scene_vec.push(scene);
                            return scene_vec
                        })
                } else {
                    return Task::new(|| { Err(
                        coffee::Error::IO(std::io::Error::new(ErrorKind::InvalidData, "ERROR: expected string describing scene path"))
                    )})
                }
            }

            scene_task.map(|scene_vec| {
                SceneStack {
                    stack: scene_vec,
                    phantom_input: PhantomData
                }
            })
        } else {
            return Task::new(|| { Err(
                coffee::Error::IO(std::io::Error::new(ErrorKind::InvalidData, "ERROR: expected Array of path strings"))
            )})
        }
    }
}

pub struct SceneStack<T: Input> {
    pub stack: Vec<Box<dyn Scene<T>>>,
    phantom_input: PhantomData<T>
}

impl<T: Input> SceneStack<T> {
    pub fn update(&mut self, ecs: &mut World) {
        let transition: SceneTransition<T>;

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

    pub fn draw(&mut self, ecs: &mut World, frame: &mut Frame, timer: &Timer) {
        if let Some(scene) = self.stack.last_mut() {
            scene.draw(ecs, frame, timer);
        } else {
            println!("ERROR: scene stack is empty");
        }
    }

    pub fn interact(&mut self, ecs: &mut World, input: &mut T, window: &mut Window) {
        if let Some(scene) = self.stack.last_mut() {
            scene.interact(ecs, input, window);
        } else {
            println!("ERROR: scene stack is empty");
        }
    }
}