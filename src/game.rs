use specs::{World, WorldExt};

use crate::scenes::scene_stack::{SceneStack, SceneStackError, SceneStackLoader, SCENE_STACK_FILE_ID};

use std::marker::PhantomData;
use std::sync::{RwLock, Arc, Mutex, PoisonError, RwLockReadGuard, RwLockWriteGuard};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, trace, debug};

use crate::game::GameError::{GameInteractError, GameDrawError, GameUpdateError, GameIsFinishedError, GameWrapperLoadError, WorldReadLockError, WorldWriteLockError};
use std::fmt::Debug;
use crate::input::Input;
use crate::loading::GenTask;
use luminance_glfw::GL33Context;
use anyhow::Result;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use crate::load::{LOAD_PATH, JSON_FILE};
use crate::scenes::SCENES_DIR;

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input + Debug>: Send {
    fn register_components(ecs: &mut World);
    // Allow user to pre-fill World with global values here
    fn load() -> GenTask<SceneStack<T>>;
    // fn load_scene_stack(ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>>;
}

// pub struct Game<T: GameWrapper<U>, U: 'static + Input + Debug> {
//     scene_stack: SceneStack<U>,
//     phantom_wrapper: PhantomData<T>,
// }

// impl<T: GameWrapper<U>, U: Input + Debug> Game<T,U> {
//     #[cfg_attr(feature="trace", instrument(skip(ecs)))]
//     pub(crate) fn load(ecs: Arc<RwLock<World>>) -> Result<Game<T,U>, GameError> {
//         #[cfg(feature="trace")]
//         debug!("ENTER: Game::load");
//         T::register_components(
//             ecs.write()
//                 .map_err(|_e| {
//                     #[cfg(feature = "trace")]
//                     error!("Failed to acquire write lock for World");
//
//                     WorldWriteLockError
//                 })?
//                 .deref_mut()
//         );
//         #[cfg(feature="trace")]
//         debug!("Components registered");
//
//         let scene_stack = T::load()
//             .execute(ecs.clone())
//             .map_err(|e| { GameWrapperLoadError { source: e } })?;
//         #[cfg(feature="trace")]
//         debug!("SceneStack loaded from GameWrapper: {:?}", scene_stack);
//
//         ecs.write()
//            .map_err(|_e| { WorldWriteLockError })?
//            .maintain();
//         #[cfg(feature="trace")]
//         debug!("World maintained after loading");
//
//         #[cfg(feature="trace")]
//         debug!("EXIT: MyGame::load");
//         Ok(Game {
//             scene_stack,
//             phantom_wrapper: PhantomData,
//         })
//     }
//
//     #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
//     pub(crate) fn draw(&mut self, ecs: Arc<RwLock<World>>) -> Result<(), GameError> {
//         #[cfg(feature="trace")]
//         debug!("ENTER: MyGame::draw");
//
//          self.scene_stack.draw(ecs)
//              .map_err(|e| {
//                  #[cfg(feature="trace")]
//                  error!("ERROR: Game failed during draw function: {:?}", e);
//
//                  GameDrawError { source: e }
//              })?;
//
//         #[cfg(feature="trace")]
//         debug!("EXIT: MyGame::draw");
//         Ok(())
//     }
//
//     #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
//     pub(crate) fn interact(&mut self, ecs: Arc<RwLock<World>>, input: &U) -> Result<(), GameError> {
//         #[cfg(feature="trace")]
//         debug!("ENTER: MyGame::interact");
//
//         self.scene_stack.interact(ecs, input)
//             .map_err(|e| {
//                 #[cfg(feature="trace")]
//                 error!("ERROR: Game failed during interact function: {:?}", e);
//
//                 GameInteractError { source: e }
//             })?;
//
//         #[cfg(feature="trace")]
//         debug!("EXIT: MyGame::interact");
//         Ok(())
//     }
//
//     #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
//     pub(crate) fn update(&mut self, ecs: Arc<RwLock<World>>) -> Result<(), GameError> {
//         #[cfg(feature="trace")]
//         debug!("ENTER: MyGame::update");
//
//         self.scene_stack.update(ecs)
//             .map_err(|e| {
//                 #[cfg(feature="trace")]
//                 error!("ERROR: Game failed during update function: {:?}", e);
//
//                 GameUpdateError { source: e }
//             })?;
//
//         #[cfg(feature="trace")]
//         trace!("EXIT: MyGame::update");
//         Ok(())
//     }
//
//     #[cfg_attr(feature="trace", instrument(skip(self, ecs)))]
//     pub(crate) fn is_finished(&self, ecs: Arc<RwLock<World>>) -> bool {
//         #[cfg(feature = "trace")]
//         trace!("ENTER: MyGame::is_finished");
//
//         let should_finish = self.scene_stack.is_finished(ecs)
//             .map_err(|e| {
//                 GameIsFinishedError {
//                     source: e
//                 }
//             }).unwrap();
//
//         #[cfg(feature = "trace")]
//         trace!("EXIT: MyGame::is_finished. is_finished: {:?}", should_finish);
//         return should_finish
//     }
// }

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Error during draw")]
    GameDrawError { source: SceneStackError },
    #[error("Error during interact")]
    GameInteractError { source: SceneStackError },
    #[error("Error during update")]
    GameUpdateError { source: SceneStackError },
    #[error("Error during is_finished")]
    GameIsFinishedError { source: SceneStackError },
    #[error("Failed to execute load function for GameWrapper")]
    GameWrapperLoadError { source: anyhow::Error },
    #[error("Failed to read World")]
    WorldReadLockError,
    #[error("Failed to write World")]
    WorldWriteLockError,
}