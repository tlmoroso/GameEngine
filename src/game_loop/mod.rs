#[cfg(feature = "trace")]
use tracing::{instrument, debug, error};

use crate::game::{GameWrapper, Game, GameError};
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use std::process::exit;
use glfw::{WindowEvent, Key, Action, Context as _};
use crate::input::Input;
use std::fmt::Debug;
use std::marker::PhantomData;
use specs::{World, WorldExt};
use luminance_windowing::WindowOpt;
use std::sync::{Arc, Mutex, RwLock};
use std::ops::DerefMut;
use thiserror::Error;
use crate::game_loop::GameLoopError::{SurfaceCreationError, ContextWriteLockError, WorldWriteLockError, GameLoadFailure, GameInteractError, GameUpdateError, GameDrawError};

#[derive(Debug)]
pub struct GameLoop<T: GameWrapper<U>, U: Input + Debug> {
    wrapper: PhantomData<T>,
    input: PhantomData<U>
}

impl<T: GameWrapper<U>, U: Input + Debug + 'static> GameLoop<T,U> {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new() -> Self {
        Self {
            wrapper: PhantomData,
            input: PhantomData
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    pub fn run(self, options: WindowOpt, name: String) -> Result<(), GameLoopError> {
        #[cfg(feature = "trace")]
        debug!("ENTER: GameLoop::run");

        let surface = GlfwSurface::new_gl33(name, options)
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("An error occurred while creating the GlfwSurface");

                SurfaceCreationError { source: e }
            })?;

        #[cfg(feature = "trace")]
        debug!("GlfwSurface created. Calling main loop");

        self.main_loop(surface)?;

        #[cfg(feature = "trace")]
        debug!("EXIT: Returned from main loop. Exiting GameLoop::run");

        Ok(())
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, surface)))]
    fn main_loop(&self, surface: GlfwSurface) -> Result<(), GameLoopError> {
        #[cfg(feature = "trace")]
        debug!("ENTER: GameLoop::main_loop");

        let context = Arc::new(RwLock::new(surface.context));
        #[cfg(feature = "trace")]
        debug!("Context created");

        let events = surface.events_rx;
        #[cfg(feature = "trace")]
        debug!("Events receiver created from surface");

        // May need to call this inside the loop to get a new buffer ever frame.
        // let back_buffer = ctxt.back_buffer().expect("back buffer");

        let mut input = U::new();
        #[cfg(feature = "trace")]
        debug!("Input Created: {:?}", input);

        let ecs = Arc::new(RwLock::new(World::new()));
        #[cfg(feature = "trace")]
        debug!("World created");

        let mut game: Game<T,U> = Game::load(ecs.clone(), context.clone())
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to load game: {:?}", e);

                GameLoadFailure { source: e }
            })?;
        #[cfg(feature = "trace")]
        debug!("Game loaded");

        #[cfg(feature = "trace")]
        debug!("Setup complete. Entering game loop.");
        loop {
            // handle events
            context.write()
                .map_err(|_e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for context");

                    ContextWriteLockError
                })?
                .window
                .glfw
                .poll_events();
            #[cfg(feature = "trace")]
            debug!("Polled events from context. Now processing them.");

            for (_thing, event) in glfw::flush_messages(&events) {
                #[cfg(feature = "trace")]
                debug!("Processing event: {:?}", event);

                // Interact or handle close/window events
                match event {
                    WindowEvent::Close
                    | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                        #[cfg(feature = "trace")]
                        debug!("Exiting game due to event. Returning without error");

                        return Ok(())
                    },
                    WindowEvent::Key(..)
                    | WindowEvent::Focus(_)
                    | WindowEvent::CursorEnter(..)
                    | WindowEvent::MouseButton(..)
                    | WindowEvent::CursorPos(..)
                    | WindowEvent::Scroll(..)
                    | WindowEvent::CharModifiers(..)
                    | WindowEvent::Char(_) => {
                        #[cfg(feature = "trace")]
                        debug!("Processing input event.");

                        input.update(event);
                        game.interact(ecs
                          .write()
                          .map_err(|_e| {
                              #[cfg(feature = "trace")]
                              error!("Failed to acquire the write lock for World");

                              WorldWriteLockError
                          })?
                          .deref_mut(), &input
                        ).map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Error occurred while running Game::interact: {:?}", e);

                            GameInteractError { source: e }
                        })?;
                        input.clear();
                    },
                    _ => ()
                }

                #[cfg(feature = "trace")]
                debug!("Events processed. Now updating game");

                // Update
                game.update(ecs
                    .write()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire write lock for World");

                        WorldWriteLockError
                    })?
                    .deref_mut()
                ).map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Error occurred while running Game::update");

                    GameUpdateError { source: e }
                })?;

                #[cfg(feature = "trace")]
                debug!("Game updated. Now calling Game::draw");

                // Draw
                game.draw(ecs
                              .write()
                              .map_err(|_e| {
                                  #[cfg(feature = "trace")]
                                  error!("Failed to acquire write lock for World");

                                  WorldWriteLockError
                              })?
                              .deref_mut(),
                          context
                              .write()
                              .map_err(|_e| {
                                  #[cfg(feature = "trace")]
                                  error!("Failed to acquire write lock for Context");

                                  ContextWriteLockError
                              })?
                              .deref_mut()
                ).map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Error occurred while running Game::draw");

                    GameDrawError { source: e }
                })?;

                #[cfg(feature = "trace")]
                debug!("Rendering complete. Swapping buffers to put new graphics on screen.");

                context.write()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire write lock for context");

                        ContextWriteLockError
                    })?
                    .window
                    .swap_buffers();

                // Exit if finished
                if game.is_finished(ecs.write()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire write lock for World");

                        WorldWriteLockError
                    })?
                    .deref_mut())
                {
                    #[cfg(feature = "trace")]
                    debug!("EXIT: GameLoop::main_loop. Game returned that it has finished. Ending game loop.");

                    return Ok(())
                }

                // Clear the input before next frame
                input.clear();
                #[cfg(feature = "trace")]
                debug!("Input cleared. Completed iteration of loop.")
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum GameLoopError {
    #[error("Failed to create GlfwSurface")]
    SurfaceCreationError { source: GlfwSurfaceError },
    #[error("Failed to write context")]
    ContextWriteLockError,
    #[error("Failed to read context")]
    ContextReadLockError,
    #[error("Failed to write World")]
    WorldWriteLockError,
    #[error("Failed to read World")]
    WorldReadLockError,
    #[error("Game failed to load")]
    GameLoadFailure { source: GameError },
    #[error("Error occurred while running Game::interact")]
    GameInteractError { source: GameError },
    #[error("Error occurred while running Game::update")]
    GameUpdateError { source: GameError },
    #[error("Error occurred while running Game::draw")]
    GameDrawError { source: GameError },
    #[error("Error occurred while running Game::is_finished")]
    GameIsFinishedError { source: GameError }
}