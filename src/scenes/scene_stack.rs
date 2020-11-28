use crate::scenes::{Scene, SceneLoader};
use crate::scenes::scene_stack::SceneStackError::{SceneStackFileLoadError, SceneStackJSONLoadError, SceneStackEmptyError, SceneStackPopError, SceneStackSwapError, SceneStackReplaceError, SceneStackClearError, SceneStackUpdateError, SceneStackDrawError, SceneStackInteractError};
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
use std::cmp::{min, max};
use std::fmt::Debug;

use serde_json::{Value, from_value};
use serde::Deserialize;

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, trace, error};

pub const SCENE_STACK_FILE_ID: &str = "scene_stack";

#[derive(Debug)]
pub enum SceneTransition<T: Input + Debug> {
    POP(usize),
    PUSH(Box<dyn Scene<T>>),
    SWAP(usize, usize),
    REPLACE(usize, Box<dyn Scene<T>>),
    CLEAR,
    NONE,
}

pub struct SceneStackLoader<T: 'static + Input + Debug> {
    scene_stack_file: String,
    scene_factory: fn(JSONLoad) -> Box<dyn SceneLoader<T>>
}

#[derive(Deserialize, Debug)]
struct SceneStackLoaderJSON {
    scenes: Vec<String>
}

impl<T: 'static + Input + Debug> SceneStackLoader<T> {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String, scene_factory: fn(JSONLoad) -> Box<dyn SceneLoader<T>>) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: SceneStackLoader::new");

        let new = Self {
            scene_stack_file: file_path,
            scene_factory
        };

        #[cfg(feature="trace")]
        trace!("EXIT: SceneStackLoader::new");

        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self,ecs, window)))]
    pub fn load(self, ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>> {
        #[cfg(feature="trace")]
        trace!("ENTER: SceneStackLoader::load");
        let json_value = map_err_return!(
            load_json(&self.scene_stack_file),
            |e| { build_task_error(
                SceneStackFileLoadError {
                    path: self.scene_stack_file.clone(),
                    var_name: stringify!(self.scene_stack_file).to_string(),
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );
        #[cfg(feature="trace")]
        trace!("Value: {:#?} successfully loaded from: {:#?}", json_value, self.scene_stack_file);

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
        #[cfg(feature="trace")]
        trace!("Value type ID: {} correctly matches SCENE_STACK_FILE_ID", json_value.load_type_id);

        let scene_paths: SceneStackLoaderJSON = map_err_return!(
            from_value(json_value.actual_value.clone()),
            |e| { build_task_error(
                SceneStackJSONLoadError {
                    value: json_value.actual_value.clone(),
                    source: e
                },
                ErrorKind::InvalidData
            )}
        );
        #[cfg(feature="trace")]
        trace!("Value: {} successfully transformed into SceneStackLoaderJSON", json_value.actual_value);

        let mut scene_task = Task::new(|| {Ok(
            Vec::new()
        )});

        for scene_path in scene_paths.scenes {
            let scene_value = map_err_return!(
                load_json(&scene_path),
                |e| { build_task_error(
                    SceneStackFileLoadError {
                        path: scene_path.clone(),
                        var_name: stringify!(scene_path).to_string(),
                        source: e
                    },
                    ErrorKind::InvalidData
                )}
            );
            #[cfg(feature="trace")]
            trace!("Scene: {:#?} successfully loaded from: {:#?}", scene_value, scene_path);

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

        let task = scene_task.map(|scene_vec| {
            SceneStack {
                stack: scene_vec,
                phantom_input: PhantomData
            }
        });
        #[cfg(feature="trace")]
        trace!("EXIT: SceneStackLoader::load");
        return task
    }
}

pub struct SceneStack<T: Input + Debug> {
    pub stack: Vec<Box<dyn Scene<T>>>,
    phantom_input: PhantomData<T>
}

impl<T: Input + Debug> SceneStack<T> {
    #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
    pub fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<(), SceneStackError> {
        #[cfg(feature="trace")]
        trace!("ENTER: SceneStack::update");

        return if let Some(scene) = self.stack.last_mut() {
            #[cfg(feature="trace")]
            trace!("Calling update on {}", scene.get_name());

            let transition = scene.update(ecs)
                .map_err(|e| {
                    SceneStackUpdateError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            trace!("Scene returned transition: {:#?}", transition);

            match transition {
                SceneTransition::POP(quantity) => {
                    for i in 0..quantity {
                        let scene = self.stack
                            .pop()
                            .ok_or_else(
                                || {
                                    #[cfg(feature="trace")]
                                    error!("Attempted to pop: ({}) more scenes than available: ({}). Failed on iteration: {}", quantity, self.stack.len(), i);

                                    SceneStackPopError {
                                        num_scenes: self.stack.len(),
                                        pop_amount: quantity
                                    }
                                }
                            )?;
                        #[cfg(feature="trace")]
                        trace!("Popped scene: {}", scene.get_name());
                    }
                    #[cfg(feature="trace")]
                    trace!("{} scenes were popped", quantity)
                }
                SceneTransition::PUSH(new_scene) => {
                    #[cfg(feature="trace")]
                    trace!("Pushed new scene: {}", new_scene.get_name());

                    self.stack.push(new_scene);
                }
                SceneTransition::SWAP(scene_1, scene_2) => {
                    if scene_1 == scene_2 {
                        #[cfg(feature="trace")]
                        trace!("Swap unnecessary because indices both equalled: {}", scene_1);
                    } else if scene_1 >= self.stack.len() {
                        #[cfg(feature="trace")]
                        error!("Invalid indices: ({}, {}) given for swap. Max index is: {}", scene_1, scene_2, self.stack.len());

                        return Err( SceneStackSwapError {
                            bad_index: scene_1,
                            length: self.stack.len()
                        })
                    } else if scene_2 >= self.stack.len() {
                        #[cfg(feature="trace")]
                        error!("Invalid indices: ({}, {}) given for swap. Max index is: {}", scene_1, scene_2, self.stack.len());

                        return Err( SceneStackSwapError {
                            bad_index: scene_2,
                            length: self.stack.len()
                        })
                    } else {
                        let max = max(scene_1, scene_2);
                        let min = min(scene_1, scene_2);
                        let max_scene = self.stack.remove(max);
                        let max_name = max_scene.get_name();

                        self.stack.insert(min, max_scene);

                        let min_scene = self.stack.remove(min + 1);
                        let min_name = min_scene.get_name();
                        self.stack.insert(max, min_scene);

                        #[cfg(feature="trace")]
                        trace!("Swapped stack positions of {} (index: {}) and {} (index: {})", max_name, max, min_name, min);
                    }
                }
                SceneTransition::REPLACE(index, new_scene) => {
                    if index >= self.stack.len() {
                        return Err( SceneStackReplaceError {
                                bad_index: index,
                                length: self.stack.len()
                            })
                    } else {
                        let new_scene_name = new_scene.get_name();
                        self.stack.insert(index, new_scene);
                        let deleted_scene = self.stack.remove(index + 1);

                        #[cfg(feature="trace")]
                        trace!("Replaced: {:#?} with {:#?}", deleted_scene.get_name(), new_scene_name);
                    }
                }
                SceneTransition::CLEAR => {
                    let stack_height = self.stack.len();
                    // Only call pop length - 1 times so one scene is left.
                    for i in 0..stack_height - 1 {
                       let deleted_scene = self.stack
                           .pop()
                           .ok_or_else(
                               || {
                                   #[cfg(feature="trace")]
                                   error!("Attempted to pop scene but received None instead. Iteration: {}. Index: {}. Original length: {}.", i, stack_height - 1 - i, stack_height);

                                   SceneStackClearError {
                                       bad_index: i,
                                       original_length: stack_height,
                                       current_length: self.stack.len()
                                   }
                               }
                           )?;

                       #[cfg(feature="trace")]
                       trace!("Clearing stack... Deleted: {} ({}/{})", deleted_scene.get_name(), i + 1, stack_height - 1);
                    }
                    let remaining_scene = self.stack
                        .first()
                        .ok_or(
                            SceneStackEmptyError {}
                        )?;

                    #[cfg(feature="trace")]
                    trace!("Cleared full scene stack except for bottom scene: {}", remaining_scene.get_name())
                }
                SceneTransition::NONE => {
                    #[cfg(feature="trace")]
                    trace!("No scene transition action was performed. Current scene: {}", scene.get_name())
                }
            };

            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::update");
            anyhow::Result::Ok(())
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty during update call");
            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::update");
            Err( SceneStackEmptyError {})
        }
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs, frame, timer)))]
    pub fn draw(&mut self, ecs: Arc<RwLock<World>>, frame: &mut Frame, timer: &Timer) -> Result<(), SceneStackError> {

        #[cfg(feature="trace")]
        trace!("ENTER: SceneStack::draw");

        return if let Some(scene) = self.stack.last_mut() {
            scene.draw(ecs, frame, timer)
                .map_err( |e| {
                    SceneStackDrawError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            trace!("Called draw on {}", scene.get_name());

            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::draw");

            Result::Ok(())
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty");

            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::draw");

            Err( SceneStackEmptyError {})
        }
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs, window)))]
    pub fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &mut T, window: &mut Window) -> Result<(), SceneStackError> {
        #[cfg(feature="trace")]
        trace!("ENTER: SceneStack::interact");

        return if let Some(scene) = self.stack.last_mut() {
            scene.interact(ecs, input, window)
                .map_err(|e| {
                    SceneStackInteractError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            trace!("Called interact on {}", scene.get_name());

            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::interact");

            Result::Ok(())
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty");

            #[cfg(feature="trace")]
            trace!("EXIT: SceneStack::interact");

            Err( SceneStackEmptyError {})
        }
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
    SceneStackEmptyError {},
    #[error("Attempted to pop {pop_amount} scenes, but only {num_scenes} are available to pop")]
    SceneStackPopError {
        num_scenes: usize,
        pop_amount: usize,
    },
    #[error("Index: {bad_index} provided for swap is out of bounds: (0..{length})")]
    SceneStackSwapError {
        bad_index: usize,
        length: usize
    },
    #[error("Index: {bad_index} provided for replacement is out of bounds: (0..{length})")]
    SceneStackReplaceError {
        bad_index: usize,
        length: usize
    },
    #[error("Error during clear of scene stack. Calling pop on iteration {bad_index} returned None even though stack had length {}")]
    SceneStackClearError {
        bad_index: usize,
        original_length: usize,
        current_length: usize

    },
    #[error("Error during call to {scene_name}.update()")]
    SceneStackUpdateError {
        scene_name: String,
        source: anyhow::Error
    },
    #[error("Error during call to {scene_name}.draw()")]
    SceneStackDrawError {
        scene_name: String,
        source: anyhow::Error
    },
    #[error("Error during call to {scene_name}.interact()")]
    SceneStackInteractError {
        scene_name: String,
        source: anyhow::Error
    }
}