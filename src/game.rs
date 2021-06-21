use coffee::load::{Task};

use specs::{World};

use crate::scenes::scene_stack::{SceneStack, SceneStackError};

use std::marker::PhantomData;
use std::sync::{RwLock, Arc};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, trace};
use crate::game::GameError::{GameInteractError, GameDrawError, GameUpdateError, GameIsFinishedError};
use std::fmt::Debug;
use crate::input::Input;

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input + Debug> {
    fn register_components(ecs: &mut World);
    // Allow user to pre-fill World with global values here
    fn load() -> Task<(World, SceneStack<T>)>;
    // fn load_scene_stack(ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>>;
}

pub struct Game<T: GameWrapper<U>, U: Input + Debug> {
    scene_stack: SceneStack<U>,
    ecs: World,
    phantom_wrapper: PhantomData<T>,
}

impl<T: GameWrapper<U>, U: Input + Debug + 'static> Game<T,U> {
    #[cfg_attr(feature="trace", instrument(skip(window)))]
    pub(crate) fn load() -> Task<Game<T,U>> {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::load");
            let task = T::load()
                .map(|(mut ecs, scene_stack)| {
                    // let mut write_ecs = ecs.write().expect("Error acquiring lock for ecs");
                    T::register_components(&mut ecs);
                    // write_ecs.maintain();
                    // std::mem::drop(write_ecs);

                    Game {
                        scene_stack,
                        ecs,
                        phantom_wrapper: PhantomData,
                    }
                });
        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::load");
        return task
    }

    #[cfg_attr(feature="trace", instrument(skip(self, frame, timer)))]
    pub(crate) fn draw(&mut self) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::draw");

        // frame.clear(Color::BLACK);
        let result = self.scene_stack.draw(&mut self.ecs);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during draw function:\n{:#?}", e);

            panic!(GameDrawError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::draw")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, window)))]
    pub(crate) fn interact(&mut self, input: &U) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::interact");

        let result = self.scene_stack.interact(&mut self.ecs, input);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during interact function:\n{:#?}", e);

            panic!(GameInteractError { source: e })
        }
        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::interact")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _window)))]
    pub(crate) fn update(&mut self) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::update");

        let result = self.scene_stack.update(&mut self.ecs);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during update function:\n{:#?}", e);

            panic!(GameUpdateError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::update");
    }

    #[cfg_attr(feature="trace", instrument(skip(self)))]
    pub(crate) fn is_finished(&self) -> bool {
        #[cfg(feature = "trace")]
        trace!("ENTER: MyGame::is_finished");

        let should_finish = self.scene_stack.is_finished()
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