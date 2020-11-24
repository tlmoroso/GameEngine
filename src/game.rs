use coffee::graphics::{Color, Frame, Window};
use coffee::load::{Task, Join, LoadingScreen};
use coffee::input::Input;
use coffee::{Game, Timer};

use specs::{World};
use specs::WorldExt;

use crate::scenes::scene_stack::SceneStack;

use std::marker::PhantomData;
use std::sync::{RwLock, Arc};

use thiserror::Error;
use crate::game::GameError::{GameDrawError, GameInteractError, GameUpdateError};

pub const GAME_FILE_ID: &str = "game";

pub trait GameWrapper<T: Input>: Game {
    // Allow user to pre-fill World with global values here
    fn load_world(_window: &Window) -> Task<World> {
        return Task::new(|| {Ok(World::new())})
    }

    fn load_scene_stack(window: &Window) -> Task<SceneStack<T>>;
}

struct MyGame<T: GameWrapper<U>, U: Input, V: LoadingScreen> {
    scene_stack: SceneStack<U>,
    ecs: Arc<RwLock<World>>,
    phantom_wrapper: PhantomData<T>,
    phantom_loading_screen: PhantomData<V>,
}

impl<T: GameWrapper<U>, U: 'static + Input, V: LoadingScreen> Game for MyGame<T,U,V> {
    type Input = U;
    type LoadingScreen = V;

    fn load(window: &Window) -> Task<MyGame<T,U,V>> {
        (
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
            })
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        frame.clear(Color::BLACK);
        let result = self.scene_stack.draw(self.ecs.clone(), frame, timer);
        if let Err(e) = result {
            panic!(GameDrawError { source: e })
        }
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        let result = self.scene_stack.interact(self.ecs.clone(), input, window);
        if let Err(e) = result {
            panic!(GameInteractError { source: e })
        }
    }

    fn update(&mut self, _window: &Window) {
        let result = self.scene_stack.update(self.ecs.clone());
        if let Err(e) = result {
            panic!(GameUpdateError { source: e })
        }
    }
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Error during draw")]
    GameDrawError { source: anyhow::Error },
    #[error("Error during interact")]
    GameInteractError { source: anyhow::Error },
    #[error("Error during update")]
    GameUpdateError { source: anyhow::Error }
}