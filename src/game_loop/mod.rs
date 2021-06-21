use crate::game::{GameWrapper, Game};
use luminance_windowing::WindowOpt;
use luminance_glfw::GlfwSurface;
use std::process::exit;
use glfw::{WindowEvent, Key, Action, Context as _};
use crate::input::Input;
use std::fmt::Debug;
use std::marker::PhantomData;
use specs::{World, WorldExt};

pub struct GameLoop<T: GameWrapper<U>, U: Input + Debug> {
    wrapper: PhantomData<T>,
    input: PhantomData<U>
}

impl<T: GameWrapper<U>, U: Input + Debug + 'static> GameLoop<T,U> {
    pub fn run(self, game: Game<T, U>, options: WindowOpt, name: String) {
        let surface = GlfwSurface::new_gl33(name, options);
        match surface {
            Ok(surface) => {
                eprintln!("graphics surface created");
                self.main_loop(surface, game);
            }

            Err(e) => {
                eprintln!("cannot create graphics surface:\n{}", e);
                exit(1);
            }
        }
    }

    fn main_loop(&self, surface: GlfwSurface, mut game: Game<T, U>) {
        let mut ctxt = surface.context;
        let events = surface.events_rx;
        // May need to call this inside the loop to get a new buffer ever frame.
        let back_buffer = ctxt.back_buffer().expect("back buffer");

        let mut input = U::new();
        let mut ecs = World::new();

        'app: loop {
            // handle events
            ctxt.window.glfw.poll_events();
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
                    | WindowEvent::Key(..)
                    | WindowEvent::Char(_) => {
                        input.update(event);
                        game.interact(&mut ecs, &input);
                    },
                    _ => ()
                }

                // Update
                game.update(&mut ecs);

                // Draw
                game.draw(&mut ecs);
                ctxt.window.swap_buffers();

                // Exit if finished
                if game.is_finished(&mut ecs) { break 'app }

                // Clear the input before next frame
                input.clear();
            }
        }
    }
}