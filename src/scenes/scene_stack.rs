use crate::scenes::{Scene, SceneLoader};
use crate::scenes::scene_stack::SceneStackError::{SceneStackFileLoadError, SceneStackJSONLoadError, SceneStackEmptyError};
use crate::load::{load_json, JSONLoad, LoadError, build_task_error};
use crate::load::LoadError::{LoadIDError};

use specs::{World};

use coffee::graphics::{Window, Frame};
use coffee::load::{Task, Join};
use coffee::Timer;
use coffee::input::Input;

use std::io::ErrorKind;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use serde_json::{Value, from_value};
use serde::Deserialize;

use thiserror::Error;

use anyhow::Result;

pub const SCENE_STACK_FILE_ID: &str = "scene_stack";

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

#[derive(Deserialize)]
struct SceneStackLoaderJSON {
    scenes: Vec<String>
}

impl<T: 'static + Input> SceneStackLoader<T> {
    pub fn new(file_path: String, scene_factory: fn(JSONLoad) -> Box<dyn SceneLoader<T>>) -> Self {
        Self {
            scene_stack_file: file_path,
            scene_factory
        }
    }

    pub fn load(self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>> {
        let json_value = map_err_return!(
            load_json(&self.scene_stack_file),
            |e| { build_task_error(
                SceneStackFileLoadError {
                    path: self.scene_stack_file,
                    var_name: stringify!(self.scene_stack_file).to_string(),
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );

        if json_value.load_type_id != SCENE_STACK_FILE_ID {
            return build_task_error(
                LoadIDError {
                    actual: json_value.load_type_id,
                    expected: SCENE_STACK_FILE_ID.to_string(),
                    json_path: self.scene_stack_file.clone()
                },
                ErrorKind::InvalidData
            )
        }

        let scene_paths: SceneStackLoaderJSON = map_err_return!(
            from_value(json_value.actual_value.clone()),
            |e| { build_task_error(
                SceneStackJSONLoadError {
                    value: json_value.actual_value,
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );

        let mut scene_task = Task::new(|| {Ok(
            Vec::new()
        )});

        for scene_path in scene_paths.scenes {
            let scene_value = map_err_return!(
                load_json(&scene_path),
                |e| { build_task_error(
                    SceneStackFileLoadError {
                        path: "".to_string(),
                        var_name: stringify!(scene_path).to_string(),
                        source: e
                    },
                    ErrorKind::InvalidData
                )}
            );

            let scene_loader = (self.scene_factory)(scene_value);
            scene_task = (
                scene_loader.load_scene(ecs.clone(), window),
                scene_task
            )
                .join()
                .map(|(scene, mut scene_vec)| {
                    scene_vec.push(scene);
                    return scene_vec
                })
        }

        scene_task.map(|scene_vec| {
            SceneStack {
                stack: scene_vec,
                phantom_input: PhantomData
            }
        })
    }
}

pub struct SceneStack<T: Input> {
    pub stack: Vec<Box<dyn Scene<T>>>,
    phantom_input: PhantomData<T>
}

impl<T: Input> SceneStack<T> {
    pub fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<()> {
        let transition: SceneTransition<T>;

        if let Some(scene) = self.stack.last_mut() {
            transition = scene.update(ecs)?;
        } else {
            return anyhow::Result::Err(anyhow::Error::new(SceneStackEmptyError {}))
        }

        match transition {
            SceneTransition::POP(_quantity) => {},
            SceneTransition::PUSH(_new_scene) => {},
            SceneTransition::SWAP(_new_scene) => {},
            SceneTransition::CLEAR => {},
            _ => {}
        };

        return anyhow::Result::Ok(())
    }

    pub fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer) -> Result<()> {
        if let Some(scene) = self.stack.last_mut() {
            scene.draw(ecs, frame, timer)?;
        } else {
            return Result::Err(anyhow::Error::new(SceneStackEmptyError {}))
        }

        return Result::Ok(())
    }

    pub fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut T, window: &mut Window) -> Result<()> {
        if let Some(scene) = self.stack.last_mut() {
            scene.interact(ecs, input, window)?;
        } else {
            return Result::Err(anyhow::Error::new(SceneStackEmptyError {}))
        }

        return Result::Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SceneStackError {
    #[error("Error loading JSON Value for SceneStackLoader from: {var_name} = {path}")]
    SceneStackFileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    },
    #[error("Error converting serde_json::value::Value into SceneStackLoaderJSON.\nExpected: Value::Array<Value::String>\nActual: {value}")]
    SceneStackJSONLoadError {
        value: Value,
        source: serde_json::error::Error
    },
    #[error("Error getting scene from stack. SceneStack is empty.")]
    SceneStackEmptyError {}
}