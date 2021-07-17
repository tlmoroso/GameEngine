use crate::game::{GameWrapper, Game};
use luminance_glfw::GlfwSurface;
use std::process::exit;
use glfw::{WindowEvent, Key, Action, Context as _};
use crate::input::Input;
use std::fmt::Debug;
use std::marker::PhantomData;
use specs::{World, WorldExt};
use luminance_windowing::WindowOpt;
use std::sync::{Arc, Mutex, RwLock};
use std::ops::DerefMut;

pub struct GameLoop<T: GameWrapper<U>, U: Input + Debug> {
    wrapper: PhantomData<T>,
    input: PhantomData<U>
}

impl<T: GameWrapper<U>, U: Input + Debug + 'static> GameLoop<T,U> {
    pub fn new() -> Self {
        Self {
            wrapper: PhantomData,
            input: PhantomData
        }
    }

    pub fn run(self, options: WindowOpt, name: String) {
        let surface = GlfwSurface::new_gl33(name, options);

        match surface {
            Ok(surface) => {
                eprintln!("graphics surface created");
                self.main_loop(surface);
            }

            Err(e) => {
                eprintln!("cannot create graphics surface:\n{}", e);
                exit(1);
            }
        }
    }

    fn main_loop(&self, surface: GlfwSurface) {
        let context = Arc::new(RwLock::new(surface.context));
        eprintln!("Context created");
        let events = surface.events_rx;
        eprintln!("Events receiver retrieved");
        // May need to call this inside the loop to get a new buffer ever frame.
        // let back_buffer = ctxt.back_buffer().expect("back buffer");

        let mut input = U::new();
        eprintln!("Input created");
        let ecs = Arc::new(RwLock::new(World::new()));
        eprintln!("ECS World created");
        let mut game: Game<T,U> = Game::load(ecs.clone(), context.clone());
        println!("game loaded");

        'app: loop {
            // handle events
            context.write().expect("Failed to lock context").window.glfw.poll_events();
            for (_thing, event) in glfw::flush_messages(&events) {
                // Interact or handle close/window events
                match event {
                    WindowEvent::Close
                    | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
                    WindowEvent::Key(..)
                    | WindowEvent::Focus(_)
                    | WindowEvent::CursorEnter(..)
                    | WindowEvent::MouseButton(..)
                    | WindowEvent::CursorPos(..)
                    | WindowEvent::Scroll(..)
                    | WindowEvent::CharModifiers(..)
                    | WindowEvent::Char(_) => {
                        input.update(event);
                        game.interact(ecs
                                          .write()
                                          .expect("Failed to lock World")
                                          .deref_mut(), &input
                        );
                    },
                    _ => ()
                }

                // Update
                game.update(ecs
                    .write()
                    .expect("Failed to lock World")
                    .deref_mut()
                );

                // Draw
                game.draw(ecs
                              .write()
                              .expect("Failed to lock World")
                              .deref_mut(),
                          context
                              .write()
                              .expect("Failed to lock context")
                              .deref_mut()
                );

                context.write().expect("Failed to lock context").window.swap_buffers();

                // Exit if finished
                if game.is_finished(ecs.write().expect("Failed to lock World").deref_mut()) { break 'app }

                // Clear the input before next frame
                input.clear();
            }
        }
    }
}