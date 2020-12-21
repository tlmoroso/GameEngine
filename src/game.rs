use coffee::graphics::{Color, Frame, Window};
use coffee::load::{Task, Join, LoadingScreen};
use coffee::input::Input;
use coffee::{Game, Timer};

use specs::{World};
use specs::WorldExt;

use crate::scenes::scene_stack::{SceneStack, SceneStackError};

use std::marker::PhantomData;
use std::sync::{RwLock, Arc};

use thiserror::Error;

#[cfg(feature="trace")]
use tracing::{instrument, error, trace};
use crate::game::GameError::{GameInteractError, GameDrawError, GameUpdateError, GameIsFinishedError};
use std::fmt::Debug;

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input + Debug> {
    // Allow user to pre-fill World with global values here
    fn load(_window: &Window) -> Task<(Arc<RwLock<World>>, SceneStack<T>)>;

    // fn load_scene_stack(ecs: Arc<RwLock<World>>, window: &Window) -> Task<SceneStack<T>>;
}

pub struct MyGame<T: GameWrapper<U>, U: Input + Debug, V: LoadingScreen> {
    scene_stack: SceneStack<U>,
    ecs: Arc<RwLock<World>>,
    phantom_wrapper: PhantomData<T>,
    phantom_loading_screen: PhantomData<V>,
}

impl<T: GameWrapper<U>, U: 'static + Input + Debug, V: LoadingScreen> Game for MyGame<T,U,V> {
    type Input = U;
    type LoadingScreen = V;

    #[cfg_attr(feature="trace", instrument(skip(window)))]
    fn load(window: &Window) -> Task<MyGame<T,U,V>> {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::load");
            let task = T::load(window)
                .map(|(ecs, scene_stack)| {
                    MyGame {
                        scene_stack,
                        ecs,
                        phantom_wrapper: PhantomData,
                        phantom_loading_screen: PhantomData
                    }
                });
        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::load");
        return task
    }

    #[cfg_attr(feature="trace", instrument(skip(self, frame, timer)))]
    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::draw");

        frame.clear(Color::BLACK);
        let result = self.scene_stack.draw(self.ecs.clone(), frame, timer);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during draw function:\n{:#?}", e);

            panic!(GameDrawError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::draw")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, window)))]
    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::interact");

        let result = self.scene_stack.interact(self.ecs.clone(), input, window);
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during interact function:\n{:#?}", e);

            panic!(GameInteractError { source: e })
        }
        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::interact")
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _window)))]
    fn update(&mut self, _window: &Window) {
        #[cfg(feature="trace")]
        trace!("ENTER: MyGame::update");

        let result = self.scene_stack.update(self.ecs.clone());
        if let Err(e) = result {
            #[cfg(feature="trace")]
            error!("ERROR: Game failed during update function:\n{:#?}", e);

            panic!(GameUpdateError { source: e })
        }

        #[cfg(feature="trace")]
        trace!("EXIT: MyGame::update");
    }

    #[cfg_attr(feature="trace", instrument(skip(self)))]
    fn is_finished(&self) -> bool {
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