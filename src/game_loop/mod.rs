#[cfg(feature = "trace")]
use tracing::{instrument, debug, error};

use crate::game::{GameWrapper, GameError};
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use std::process::exit;
use glfw::{WindowEvent, Key, Action, Context, WindowMode, SwapInterval};
use crate::input::Input;
use std::fmt::Debug;
use std::marker::PhantomData;
use specs::{World, WorldExt};
use std::sync::{Arc, RwLock, LockResult, RwLockReadGuard};
use std::ops::DerefMut;
use thiserror::Error;
use crate::game_loop::GameLoopError::*;
use std::time::{Instant, Duration};
use std::thread::sleep;
use crate::graphics::Context as SyncContext;
use crate::threading::{Pool, ThreadError};
use crate::threading::ThreadError::{PoolReadLockError, ThreadPoolError};
use crate::scenes::scene_stack::{SceneStackError, SceneStack};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use parking_lot::{Condvar, Mutex};
use once_cell::sync::OnceCell;
use std::sync::mpsc::{Receiver, channel};
use crossbeam::thread::{scope, ScopedJoinHandle};
use crossbeam::sync::Parker;

const NUM_THREADS: usize = 8;
const FPS: u64 = 60;
const ONE_SEC_MICROS: u64 = 1_000_000;
const FRAME_TIME_MICROS: Duration = Duration::from_micros(ONE_SEC_MICROS/FPS);

struct EventsReceiver(Arc<RwLock<Option<Receiver<(f64, WindowEvent)>>>>);

unsafe impl Sync for EventsReceiver {}

unsafe impl Send for EventsReceiver {}

impl Default for EventsReceiver {
    fn default() -> Self {
        EventsReceiver {
            0: Arc::new(RwLock::new(None))
        }
    }
}

impl EventsReceiver {
    pub fn new(rx: Receiver<(f64, WindowEvent)>) -> Self {
        Self {
            0: Arc::new(RwLock::new(Some(rx)))
        }
    }

    pub fn set(&self, rx: Receiver<(f64, WindowEvent)>) -> Result<(), GameLoopError> {
        let mut receiver = self.0.write()
            .map_err(|_e| {
                #[cfg(feature = "trace")]
                error!("Failed to acquire write lock for events receiver");

                EventsReceiverWriteError
            })?;

        *receiver = Some(rx);

        Ok(())
    }

    pub fn get(&self) -> LockResult<RwLockReadGuard<'_, Option<Receiver<(f64, WindowEvent)>>>> {
        self.0.read()
    }
}

#[derive(Debug)]
pub struct GameLoop<T: GameWrapper<U>, U: Input + Debug> {
    wrapper: PhantomData<T>,
    input: PhantomData<U>
}

impl<T: 'static + GameWrapper<U>, U: Input + Debug + 'static> GameLoop<T,U> {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new() -> Self {
        Self {
            wrapper: PhantomData,
            input: PhantomData
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    pub fn run(self, /*options: WindowOpt,*/ name: String) -> Result<(), GameLoopError> {
        #[cfg(feature = "trace")]
        debug!("ENTER: GameLoop::run");

        self.main_loop(/*surface, options,*/ name)?;

        #[cfg(feature = "trace")]
        debug!("EXIT: Returned from main loop. Exiting GameLoop::run");

        Ok(())
    }

    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    fn main_loop(&self/*, surface: GlfwSurface*/, /*options: WindowOpt,*/ name: String) -> Result<(), GameLoopError> {
        let surface = GlfwSurface::new(|glfw| {
            let (mut window, events) = glfw.create_window(960, 540, "Hello this is window", WindowMode::Windowed)
                .expect("Failed to create GLFW window.");

            window.make_current();
            window.set_all_polling(true);
            glfw.set_swap_interval(SwapInterval::Sync(1));

            Ok((window, events))
        })
        .map_err(|_: GlfwSurfaceError<anyhow::Error>| {
            #[cfg(feature = "trace")]
            error!("An error occurred while creating the GlfwSurface");

            SurfaceCreationError
        })?;

        #[cfg(feature = "trace")]
        debug!("GlfwSurface created. Calling main loop");

        let context = SyncContext::new(surface.context);
        #[cfg(feature = "trace")]
        debug!("Context created");

        let mut ecs = Arc::new(RwLock::new(World::new()));
        #[cfg(feature = "trace")]
        debug!("World created");

        let mut ecs_write = ecs.write()
            .map_err(|_e| {
                #[cfg(feature = "trace")]
                error!("Failed to acquire write lock while inserting thread pool");

                WorldWriteLockError
            }).expect("Failed to acquire write lock for World.");

        T::register_components(ecs_write.deref_mut());
        #[cfg(feature="trace")]
        debug!("Components registered");

        drop(ecs_write);

        #[cfg(feature = "trace")]
        debug!("Thread Pool created");

        let scene_stack: OnceCell<SceneStack<U>> = OnceCell::new();

        let quit = AtomicBool::new(false);

        let init_loops = Arc::new((Mutex::new(false), Condvar::new()));
        let init_update_loop = init_loops.clone();
        let init_draw_loop = init_loops.clone();
        let init_finish_loop = init_loops.clone();
        let init_interact_loop = init_loops.clone();

        let rx = EventsReceiver::default();
        // let (events_rx_tx, events_rx_rx) = channel();

        let _game_result: Result<Result<(), GameLoopError>, Box<dyn std::any::Any + Send>> = scope(|scope| {
            let is_finished_handle: ScopedJoinHandle<Result<(), GameLoopError>> = scope.spawn(|_| {
                let (start_lock, condvar) = &*init_finish_loop;
                let mut start = start_lock.lock();
                if !*start {
                    condvar.wait(&mut start);
                }

                drop(start);

                loop {
                    // Exit if finished
                    if scene_stack.get()
                        .ok_or_else(|| {
                            #[cfg(feature = "trace")]
                            error!("Failed to get SceneStack value when it was expected to exist");

                            SceneStackGetError
                        })?
                        .is_finished(ecs.clone())
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Calling is_finished resulted in an error: {:?}", e);

                            GameIsFinishedError { source: e }
                        })?
                    {
                        #[cfg(feature = "trace")]
                        debug!("EXIT: GameLoop::main_loop. Game returned that it has finished. Ending game loop.");

                        quit.store(true, Relaxed);
                        return Ok(())
                    }
                    // Doesn't need to be synced to anything for now. Just check occasionally.
                    sleep(FRAME_TIME_MICROS);
                }
            });

            let update_handle: ScopedJoinHandle<Result<(), GameLoopError>> = scope.spawn(|_| {
                let (start_lock, condvar) = &*init_update_loop;
                let mut start = start_lock.lock();
                if !*start {
                    condvar.wait(&mut start);
                }

                drop(start);

                loop {
                    let start = Instant::now();

                    // Update
                    scene_stack.get()
                        .ok_or_else(|| {
                            #[cfg(feature = "trace")]
                            error!("Failed to get SceneStack value when it was expected to exist");

                            SceneStackGetError
                        })?
                        .update(ecs.clone())
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Error occurred while running Game::update");

                            GameUpdateError { source: e }
                        })?;

                    if quit.load(Relaxed) { return Ok(()) }

                    /*
                * TODO: Will need to change this to calculate delta and pass delta time to update function
                * instead of wasting time.
                */
                    let sleep_time = FRAME_TIME_MICROS.saturating_sub(start.elapsed());
                    #[cfg(feature = "trace")]
                    debug!("Update complete. Sleeping for {:?} microseconds", sleep_time.as_micros());

                    sleep(sleep_time);
                }
            });

            let draw_handle: ScopedJoinHandle<Result<(), GameLoopError>> = scope.spawn(|s| {
                let ecs = ecs.clone();

                // events_rx_tx.send(surface.events_rx)
                //     .map_err(|_e| {
                //         #[cfg(feature = "trace")]
                //         error!("Failed to send events receiver to interact thread.");
                //
                //         ChannelSendError
                //     })?;
                // rx.set(surface.events_rx);

                #[cfg(feature = "trace")]
                debug!("Events receiver created from surface");

                let mut ecs_write = ecs.write()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire write lock while inserting context");

                        WorldWriteLockError
                    })?;

                ecs_write.insert(context);

                drop(ecs_write);

                scene_stack.set(T::load()
                    .execute(ecs.clone())
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to load Scene Stack from GameWrapper load function.");

                        GameWrapperLoadError { source: e }
                    })?
                ).map_err(|_| {
                    #[cfg(feature = "trace")]
                    error!("Failed to set SceneStack Value");

                    SceneStackSetError
                });

                #[cfg(feature="trace")]
                debug!("SceneStack loaded from GameWrapper: {:?}", scene_stack);

                let mut ecs_write = ecs.write()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire write lock while inserting thread pool");

                        WorldWriteLockError
                    })?;

                ecs_write.maintain();
                #[cfg(feature="trace")]
                debug!("World maintained after loading");

                drop(ecs_write);

                let (start_lock, condvar) = &*init_draw_loop;
                let mut start = start_lock.lock();
                *start = true;
                condvar.notify_all();

                drop(start);

                loop {
                    let start = Instant::now();

                    scene_stack.get()
                        .ok_or_else(|| {
                            #[cfg(feature = "trace")]
                            error!("Failed to get SceneStack value when it was expected to exist");

                            SceneStackGetError
                        })?
                        .draw(ecs.clone())
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Error occurred while running Game::draw");

                            GameDrawError { source: e }
                        })?;

                    #[cfg(feature = "trace")]
                    debug!("Rendering complete. Swapping buffers to put new graphics on screen.");

                    let ecs = ecs.read()
                        .map_err(|e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to acquire read lock for World while swapping buffers");

                            WorldReadLockError
                        })?;

                    let context = ecs.fetch::<SyncContext>();

                    context.0
                        .write()
                        .map_err(|_e| {
                            #[cfg(feature = "trace")]
                            error!("Failed to acquire write lock for context");

                            ContextWriteLockError
                        })?
                        .window
                        .swap_buffers();

                    if quit.load(Relaxed) { return Ok(()) }

                    let sleep_time = FRAME_TIME_MICROS.saturating_sub(start.elapsed());
                    #[cfg(feature = "trace")]
                    debug!("Draw complete. Sleeping for {:?} microseconds", sleep_time.as_micros());

                    sleep(sleep_time);
                }
            });

            // let interact_handle: ScopedJoinHandle<Result<(), GameLoopError>> = scope.spawn(|_| {
                let (start_lock, condvar) = &*init_interact_loop;
                let mut start = start_lock.lock();
                if !*start {
                    condvar.wait(&mut start);
                }

                let ecs = ecs.clone();

                let events_rx_guard = rx.get()
                    .map_err(|_e| {
                        #[cfg(feature = "trace")]
                        error!("Failed to acquire read lock for events receiver");

                        EventsReceiverReadError
                    })?;

                let events_rx = events_rx_guard
                    .as_ref()
                    .ok_or_else(|| {
                        #[cfg(feature = "trace")]
                        error!("Receiver was not initalized yet when accessed.");

                        EventsReceieverInitError
                    })?;

                // let events_rx = events_rx_rx.recv()
                //     .map_err(|_e| {
                //         #[cfg(feature = "trace")]
                //         error!("Failed to receive events_rx from channel.");
                //
                //         ChannelReceiveError
                //     })?;

                let mut input = U::new();
                #[cfg(feature = "trace")]
                debug!("Input Created: {:?}", input);

                // // Do this to convince compiler to move events variable instead of trying to borrow across threads.
                // let events_rx = events;

                loop {
                    let start = Instant::now();

                    // for (_thing, event) in events_rx.iter() {
                    for (_thing, event) in glfw::flush_messages(events_rx) {
                        println!("New event from input");

                        #[cfg(feature = "trace")]
                        debug!("Processing event: {:?}", event);

                        // Interact or handle close/window events
                        match event {
                            WindowEvent::Close
                            | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                                #[cfg(feature = "trace")]
                                debug!("Exiting game due to event. Returning without error");
                                println!("Exiting game");
                                // return;
                                std::process::exit(255);
                            },
                            WindowEvent::Key(..)
                            | WindowEvent::Focus(_)
                            | WindowEvent::CursorEnter(..)
                            | WindowEvent::MouseButton(..)
                            | WindowEvent::CursorPos(..)
                            | WindowEvent::Scroll(..)
                            | WindowEvent::CharModifiers(..)
                            | WindowEvent::Char(_) => {
                                println!("Handling window event");

                                #[cfg(feature = "trace")]
                                debug!("Processing input event.");

                                input.update(event);

                                println!("Input is updated: {:?}", input);

                                scene_stack.get()
                                    .ok_or_else(|| {
                                        #[cfg(feature = "trace")]
                                        error!("Failed to get SceneStack value when it was expected to exist");

                                        SceneStackGetError
                                    })?
                                    .interact(ecs.clone(), &input)
                                    .map_err(|e| {
                                        #[cfg(feature = "trace")]
                                        error!("Error occurred while running Game::interact: {:?}", e);

                                        GameInteractError { source: e }
                                    })?;

                                println!("Scene Stack interact done");

                                input.clear();

                                println!("Input cleared: {:?}", input);
                            },
                            _ => ()
                        };

                        if quit.load(Relaxed) {
                            let update_res = update_handle.join();
                            let draw_res = draw_handle.join();
                            let is_finished_res = is_finished_handle.join();

                            println!("Thread results: Update={:?}, Draw={:?}, IsFinished={:?}", update_res, draw_res, is_finished_res);

                            return Ok(())
                        }

                        // Sleep for the remaining time for this frame
                        let sleep_time = FRAME_TIME_MICROS.saturating_sub(start.elapsed());
                        #[cfg(feature = "trace")]
                        debug!("Interact complete. Sleeping for {:?} microseconds", sleep_time.as_micros());

                        sleep(sleep_time);
                    }
                }
            // });

            // let update_res = update_handle.join();
            // let draw_res = draw_handle.join();
            // let interact_res = interact_handle.join();
            // let is_finished_res = is_finished_handle.join();
            //
            // println!("Thread results: Update={:?}, Draw={:?}, Interact={:?}, IsFinished={:?}", update_res, draw_res, interact_res, is_finished_res);
            //
            // return ()
        });

        return Ok(())
    }
}

#[derive(Error, Debug)]
pub enum GameLoopError {
    #[error("Failed to create GlfwSurface")]
    SurfaceCreationError,
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
    GameInteractError { source: SceneStackError },
    #[error("Error occurred while running Game::update")]
    GameUpdateError { source: SceneStackError },
    #[error("Error occurred while running Game::draw")]
    GameDrawError { source: SceneStackError },
    #[error("Error occurred while running Game::is_finished")]
    GameIsFinishedError { source: SceneStackError },
    #[error("Error occurred with Threading Object")]
    ThreadingError {source: ThreadError },
    #[error("Failed to execute load function for GameWrapper")]
    GameWrapperLoadError { source: anyhow::Error },
    #[error("Error occurred while setting SceneStack value")]
    SceneStackSetError,
    #[error("SceneStack value was not set when expected to")]
    SceneStackGetError,
    #[error("Failed to acquire write lock for events receiver")]
    EventsReceiverWriteError,
    #[error("Failed to acquire read lock for events receiver")]
    EventsReceiverReadError,
    #[error("Event Receiver was not initialized when accessed.")]
    EventsReceieverInitError,
    #[error("Failed to send across channel")]
    ChannelSendError,
    #[error("Failed to receive across channel")]
    ChannelReceiveError,
}