use specs::{World, WorldExt};

use crate::scenes::scene_stack::{SceneStack, SceneStackError, SceneStackLoader, SCENE_STACK_FILE_ID};

use std::marker::PhantomData;
use std::sync::{RwLock, Arc, Mutex};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, trace};
use crate::game::GameError::{GameInteractError, GameDrawError, GameUpdateError, GameIsFinishedError};
use std::fmt::Debug;
use crate::input::Input;
use crate::loading::DrawTask;
use luminance_glfw::GL33Context;
use anyhow::Result;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use crate::load::{LOAD_PATH, JSON_FILE};
use crate::scenes::SCENES_DIR;

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input + Debug> {
    fn register_components(ecs: &mut World);
    // Allow user to pre-fill World with global values here
    fn load() -> DrawTask<SceneStack<T>>;
    // fn load_scene_stack(ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>>;
}

pub struct Game<T: GameWrapper<U>, U: 'static + Input + Debug> {
    scene_stack: SceneStack<U>,
    phantom_wrapper: PhantomData<T>,
}

impl<T: GameWrapper<U>, U: Input + Debug> Game<T,U> {
    #[cfg_attr(feature="trace", instrument(skip(window)))]
    pub(crate) fn load(ecs: Arc<RwLock<World>>, context: Arc<RwLock<GL33Context>>) -> Game<T,U> {
        #[cfg(feature="trace")]
        trace!("ENTER: Game::load");
        T::register_components(ecs.write().expect("Failed to lock World.").deref_mut());
        eprintln!("Components registered");

        let scene_stack = T::load()
            .execute((ecs.clone(), context))
            .expect("Failed to load SceneStack from GameWrapper");
        eprintln!("Scene Stack loaded");

        ecs.write()
           .expect("Failed to lock World")
           .maintain();
        eprintln!("World maintained");

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::load");
        Game {
            scene_stack,
            phantom_wrapper: PhantomData,
        }
    }

    #[cfg_attr(feature="trace", instrument(skip(self, frame, timer)))]
    pub(crate) fn draw(&mut self, ecs: &mut World, context: &mut GL33Context) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::draw");

        let result = self.scene_stack.draw(ecs, context);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during draw function:\n{:#?}", e);

            panic!("{:?}", GameDrawError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::draw")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, window)))]
    pub(crate) fn interact(&mut self, ecs: &mut World, input: &U) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::interact");

        let result = self.scene_stack.interact(ecs, input);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during interact function:\n{:#?}", e);

            panic!("{:?}", GameInteractError { source: e })
        }
        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::interact")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _window)))]
    pub(crate) fn update(&mut self, ecs: &mut World) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::update");

        let result = self.scene_stack.update(ecs);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during update function:\n{:#?}", e);

            panic!("{:?}", GameUpdateError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::update");
    }

    #[cfg_attr(feature="trace", instrument(skip(self)))]
    pub(crate) fn is_finished(&self, ecs: &mut World) -> bool {
        #[cfg(feature = "trace")]
        trace!("ENTER: MyGame::is_finished");

        let should_finish = self.scene_stack.is_finished(ecs)
            .map_err(|e| {
                GameIsFinishedError {
                    source: e
                }
            }).unwrap();

        #[cfg(feature = "trace")]
        trace!("EXIT: MyGame::is_finished");
        return should_finish
    }
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Error during draw")]
    GameDrawError { source: SceneStackError },
    #[error("Error during interact")]
    GameInteractError { source: SceneStackError },
    #[error("Error during update")]
    GameUpdateError { source: SceneStackError },
    #[error("Error during is_finished")]
    GameIsFinishedError { source: SceneStackError }
}