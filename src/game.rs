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
use crate::game::GameError::{GameInteractError, GameDrawError, GameUpdateError};
use std::fmt::Debug;

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input + Debug>: Game {
    // Allow user to pre-fill World with global values here
    fn load_world(_window: &Window) -> Task<World> {
        return Task::new(|| {Ok(World::new())})
    }

    fn load_scene_stack(window: &Window) -> Task<SceneStack<T>>;
}

struct MyGame<T: GameWrapper<U>, U: Input + Debug, V: LoadingScreen> {
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
        let task = (
            T::load_world(window),
            T::load_scene_stack(window),
        )
            .join()
            .map(|(ecs, scene_stack)| {
                MyGame {
                    scene_stack,
                    ecs: Arc::new(RwLock::new(ecs)),
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
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Error during draw")]
    GameDrawError { source: SceneStackError },
    #[error("Error during interact")]
    GameInteractError { source: SceneStackError },
    #[error("Error during update")]
    GameUpdateError { source: SceneStackError }
}