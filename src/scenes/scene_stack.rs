use crate::scenes::{Scene, SceneLoader};
use crate::scenes::scene_stack::SceneStackError::{SceneStackEmptyError, SceneStackPopError, SceneStackSwapError, SceneStackReplaceError, SceneStackClearError, SceneStackUpdateError, SceneStackDrawError, SceneStackInteractError, SceneStackIsFinishedError, SceneStackDeserializationError, SceneStackFactoryError, StackReadLockError, StackWriteLockError};
use crate::load::{load_json, JSONLoad, LoadError, load_deserializable_from_file};

use specs::World;

use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::cmp::{min, max};
use std::fmt::Debug;

use serde_json::{Value};
use serde::Deserialize;

use thiserror::Error;
use anyhow::Result;

#[cfg(feature="trace")]
use tracing::{instrument, trace, error, debug};

use crate::input::Input;
use crate::loading::{DrawTask, GenTask};
use luminance_glfw::GL33Context;
use crate::scenes::scene_stack::SceneStackLoaderError::{JSONDeserializeFromFileError, JSONLoadFromFileError, SceneFactoryError, SceneLoadError};

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

#[derive(Debug, Clone)]
pub struct SceneStackLoader<T: Input + Debug> {
    scene_stack_file: String,
    scene_factory: fn(JSONLoad) -> Result<Box<dyn SceneLoader<T>>>
}

#[derive(Deserialize, Debug, Clone)]
struct SceneStackLoaderJSON {
    scene_paths: Vec<String>
}

impl<T: 'static + Input + Debug> SceneStackLoader<T> {
    #[cfg_attr(feature="trace", instrument(skip(scene_factory)))]
    pub fn new(file_path: String, scene_factory: fn(JSONLoad) -> Result<Box<dyn SceneLoader<T>>>) -> Self {
        let new = Self {
            scene_stack_file: file_path,
            scene_factory
        };

        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self)))]
    pub fn load(&self) -> GenTask<SceneStack<T>> {
        // Attempts to not bring self into closure.
        let path = self.scene_stack_file.clone();
        let scene_factory = self.scene_factory;

        let task = GenTask::new(move |ecs| {
            let scene_stack_json: SceneStackLoaderJSON = load_deserializable_from_file(&path, SCENE_STACK_FILE_ID)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to deserialize JSON file: ({:?}) into Scene Stack JSON object", path.clone());

                    JSONDeserializeFromFileError {
                        source: e,
                        path: path.clone()
                    }
                })?;

            let mut scene_vec = Vec::new();
            #[cfg(feature = "trace")]
            debug!("SceneStack json deserialized: ({:?}). Loading scenes", scene_stack_json.clone());

            for scene_path in scene_stack_json.scene_paths {
                #[cfg(feature = "trace")]
                debug!("Loading Scene: {:?}", scene_path);

                let scene_value = load_json(&scene_path)
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        debug!("Failed to create JSONLoad object from scene file: {:?}", scene_path);

                        JSONLoadFromFileError {
                            source: e,
                            path: scene_path.clone()
                        }
                    })?;

                let scene_loader = (scene_factory)(scene_value.clone())
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("An error occurred while passing the JSON value: ({:?}) for a scene to the scene_factory", scene_value);

                        SceneFactoryError {
                            source: e,
                            scene_json: scene_value.clone()
                        }
                    })?;

                let scene = scene_loader.load_scene()
                    .execute(ecs.clone())
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("An error occurred while loading the scene: ({:?})", e);

                        SceneLoadError {
                            source: e
                        }
                    })?;


                #[cfg(feature = "trace")]
                debug!("Scene loaded: {:?}", scene.get_name());

                scene_vec.push(scene);
            }

            #[cfg(feature = "trace")]
            debug!("Returning SceneStack from Task");

            Ok(SceneStack {
                stack: RwLock::new(scene_vec),
                phantom_input: PhantomData::default()
            })
        });

        return task;
    }
}

#[derive(Debug)]
pub struct SceneStack<T: Input + Debug> {
    pub stack: RwLock<Vec<Box<dyn Scene<T>>>>,
    pub phantom_input: PhantomData<T>
}

unsafe impl<T: Input + Debug> Send for SceneStack<T> {}

unsafe impl<T: Input + Debug> Sync for SceneStack<T> {}

impl<T: Input + Debug> SceneStack<T> {
    #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
    pub fn update(&self, ecs: Arc<RwLock<World>>) -> Result<(), SceneStackError> {
        let mut transition = SceneTransition::NONE;

        let stack = self.stack.read()
            .map_err(|_e| {
                #[cfg(feature="trace")]
                error!("Failed to acquire read lock for stack while updating scene.");

                StackReadLockError
            })?;

        if let Some(scene) = stack.last() {
            #[cfg(feature = "trace")]
            debug!("Calling update on {}", scene.get_name());

            transition = scene.update(ecs)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("An occurred while calling update on scene: {:?}", scene.get_name());

                    SceneStackUpdateError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature = "trace")]
            trace!("Scene returned: {:?}", transition);
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty during update call");

            return Err(SceneStackEmptyError {});
        }

        drop(stack);
        #[cfg(feature = "trace")]
        debug!("Dropped read lock guard for stack.");


        let mut stack = self.stack.write()
            .map_err(|_e| {
                #[cfg(feature="trace")]
                error!("Failed to acquire write lock for stack while updating scene stack.");

                StackWriteLockError
            })?;

        #[cfg(feature = "trace")]
        debug!("Acquired write lock guard for stack.");

        match transition {
            SceneTransition::POP(quantity) => {
                for _i in 0..quantity {
                    let _scene = stack
                        .pop()
                        .ok_or_else(
                            || {
                                #[cfg(feature="trace")]
                                error!("Attempted to pop: {} scenes. More scenes than available: ({}). Failed on iteration: {}", quantity, stack.len(), _i);

                                SceneStackPopError {
                                    num_scenes: stack.len(),
                                    pop_amount: quantity
                                }
                            }
                        )?;
                    #[cfg(feature="trace")]
                    debug!("Popped scene: {}", _scene.get_name());
                }
                #[cfg(feature="trace")]
                debug!("{} scenes were popped", quantity)
            },
            SceneTransition::PUSH(new_scene) => {
                #[cfg(feature="trace")]
                debug!("Pushed new scene: {}", new_scene.get_name());

                stack.push(new_scene);
            },
            SceneTransition::SWAP(scene_1, scene_2) => {
                if scene_1 == scene_2 {
                    #[cfg(feature="trace")]
                    debug!("Swap unnecessary because indices both equalled: {}", scene_1);
                } else if scene_1 >= stack.len() {
                    #[cfg(feature="trace")]
                    error!("Invalid indices: ({}, {}) given for swap. Max index is: {}", scene_1, scene_2, stack.len());

                    return Err( SceneStackSwapError {
                        bad_index: scene_1,
                        length: stack.len()
                    })
                } else if scene_2 >= stack.len() {
                    #[cfg(feature="trace")]
                    error!("Invalid indices: ({}, {}) given for swap. Max index is: {}", scene_1, scene_2, stack.len());

                    return Err( SceneStackSwapError {
                        bad_index: scene_2,
                        length: stack.len()
                    })
                } else {
                    let max = max(scene_1, scene_2);
                    let min = min(scene_1, scene_2);
                    let max_scene = stack.remove(max);
                    let _max_name = max_scene.get_name();

                    stack.insert(min, max_scene);

                    let min_scene = stack.remove(min + 1);
                    let _min_name = min_scene.get_name();
                    stack.insert(max, min_scene);

                    #[cfg(feature="trace")]
                    debug!("Swapped stack positions of {} (index: {}) and {} (index: {})", _max_name, max, _min_name, min);
                }
            },
            SceneTransition::REPLACE(index, new_scene) => {
                if index >= stack.len() {
                    return Err( SceneStackReplaceError {
                            bad_index: index,
                            length: stack.len()
                        })
                } else {
                    let _new_scene_name = new_scene.get_name();
                    stack.insert(index, new_scene);
                    let _deleted_scene = stack.remove(index + 1);

                    #[cfg(feature="trace")]
                    debug!("Replaced: {:#?} with {:#?}", _deleted_scene.get_name(), _new_scene_name);
                }
            },
            SceneTransition::CLEAR => {
                let stack_height = stack.len();
                // Only call pop length - 1 times so one scene is left.
                for i in 0..stack_height - 1 {
                   let _deleted_scene = stack.pop()
                       .ok_or_else(
                           || {
                               #[cfg(feature="trace")]
                               error!("Attempted to pop scene but received None instead. Iteration: {}. Index: {}. Original length: {}.", i, stack_height - 1 - i, stack_height);

                               SceneStackClearError {
                                   bad_index: i,
                                   original_length: stack_height,
                                   current_length: stack.len()
                               }
                           }
                       )?;

                   #[cfg(feature="trace")]
                   debug!("Clearing stack... Deleted: {} ({}/{})", _deleted_scene.get_name(), i + 1, stack_height - 1);
                }
                let _remaining_scene = stack.first().ok_or(SceneStackEmptyError {})?;

                #[cfg(feature="trace")]
                debug!("Cleared full scene stack except for bottom scene: {}", _remaining_scene.get_name())
            },
            SceneTransition::NONE => {
                let _scene = stack.last().ok_or(SceneStackEmptyError {})?;

                #[cfg(feature="trace")]
                debug!("No scene transition action was performed. Current scene: {}", _scene.get_name())
            }
        };

        anyhow::Result::Ok(())
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
    pub fn draw(&self, ecs: Arc<RwLock<World>>) -> Result<(), SceneStackError> {
        return if let Some(scene) = self.stack.read().map_err(|_e| StackReadLockError)?.last() {
            scene.draw(ecs)
                .map_err( |e| {
                    #[cfg(feature = "trace")]
                    error!("An error occurred while calling Scene::draw. Error: ({:?}). Scene: {:?}", e, scene.get_name());

                    SceneStackDrawError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            debug!("Called draw on {}", scene.get_name());

            Result::Ok(())
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty");

            Err( SceneStackEmptyError {})
        }
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
    pub fn interact(&self, ecs: Arc<RwLock<World>>, input: &T) -> Result<(), SceneStackError> {
        return if let Some(scene) = self.stack.read().map_err(|_e| StackReadLockError )?.last() {
            scene.interact(ecs, input)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("An error occurred while calling Scene::interact. Error: ({:?}). Scene: {:?}", e, scene.get_name());

                    SceneStackInteractError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            debug!("Called interact on {}", scene.get_name());

            Result::Ok(())
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty");

            Err( SceneStackEmptyError {})
        }
    }

    #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
    pub fn is_finished(&self, ecs: Arc<RwLock<World>>) -> Result<bool, SceneStackError> {
        return if let Some(scene) = self.stack.read().map_err(|_e| StackReadLockError)?.last() {
            let should_finish = scene.is_finished(ecs)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("An error occurred while calling Scene::is_finished. Error: ({:?}). Scene: {:?}", e, scene.get_name());

                    SceneStackIsFinishedError {
                        scene_name: scene.get_name(),
                        source: e
                    }
                })?;

            #[cfg(feature="trace")]
            debug!("Called is_finished on {}. Received: {}", scene.get_name(), should_finish);

            Ok(should_finish)
        } else {
            #[cfg(feature="trace")]
            error!("SceneStack was empty");

            Err(SceneStackEmptyError {})
        }
    }
}

#[derive(Error, Debug)]
pub enum SceneStackLoaderError {
    #[error("Failed to deserialize Scene Stack JSON from file at {path:?}")]
    JSONDeserializeFromFileError {
        source: LoadError,
        path: String
    },
    #[error("Failed to create LoadJSON object from file at {path:?}")]
    JSONLoadFromFileError {
        source: LoadError,
        path: String
    },
    #[error("The scene factory failed to detect this scene JSON value: {scene_json:?}")]
    SceneFactoryError {
        source: anyhow::Error,
        scene_json: JSONLoad
    },
    #[error("Failed to load scene")]
    SceneLoadError {
        source: anyhow::Error
    }
}

#[derive(Error, Debug)]
pub enum SceneStackError {
    #[error("Error trying to get scene loader from scene factory multiplexer function when passing value: {scene_value:?}")]
    SceneStackFactoryError {
        scene_value: JSONLoad,
        source: anyhow::Error
    },
    #[error("Error loading SceneStackLoaderJSON from: {path}")]
    SceneStackDeserializationError {
        path: String,
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
    },
    #[error("Error during call to {scene_name}.is_finished()")]
    SceneStackIsFinishedError {
        scene_name: String,
        source: anyhow::Error
    },
    #[error("Failed to acquire stack's read lock.")]
    StackReadLockError,
    #[error("Failed to acquire stack's write lock.")]
    StackWriteLockError,
}