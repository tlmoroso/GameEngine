use crate::input::mouse::{Mouse, Button, CursorPosition, WheelMovement};
use crate::input::keyboard::{KeyBoard, Key};
use std::collections::{HashMap, HashSet};
use crate::input::Input;
use glfw::WindowEvent;

#[cfg(feature = "trace")]
use tracing::{debug, error, warn, instrument};

#[derive(Debug, Clone)]
pub struct MultiInput {
    mouse: Mouse,
    keyboard: KeyBoard,
    pub held_buttons: HashMap<Button, CursorPosition>,
    pub held_keys: HashSet<Key>
}

impl MultiInput {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_cursor_position(&self) -> CursorPosition {
        return self.mouse.get_cursor_position()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_wheel_movement(&self) -> WheelMovement {
        return self.mouse.get_wheel_movement()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn is_cursor_owned(&self) -> bool {
        return self.mouse.is_cursor_owned()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn is_cursor_within_window(&self) -> bool {
        return self.mouse.is_cursor_within_window()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_clicked_buttons(&self) -> &HashMap<Button, CursorPosition> {
        self.mouse.get_clicked_buttons()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_held_buttons(&self) -> &HashMap<Button, CursorPosition> {
        &self.held_buttons
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_released_buttons(&self) -> &HashMap<Button, CursorPosition>  {
        self.mouse.get_released_buttons()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_pressed_keys(&self) -> &HashSet<Key> {
        self.keyboard.get_pressed_keys()
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_held_keys(&self) -> &HashSet<Key> {
        &self.held_keys
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn get_released_keys(&self) -> &HashSet<Key> {
        self.keyboard.get_released_keys()
    }
}

impl Input for MultiInput {
    #[cfg_attr(feature = "trace", instrument)]
    fn new() -> Self {
        let new = Self {
            mouse: Mouse::new(),
            keyboard: KeyBoard::new(),
            held_buttons: HashMap::new(),
            held_keys: HashSet::new()
        };

        #[cfg(feature = "trace")]
        debug!("Created new empty MultiInput: {:?}", new.clone());

        return new
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn update(&mut self, event: WindowEvent) {
        #[cfg(feature = "trace")]
        debug!("Matching on window event: {:?}", event);

        match event {
            WindowEvent::Key(..)
            | WindowEvent::Char(_)
            | WindowEvent::CharModifiers(..) => {
                #[cfg(feature = "trace")]
                debug!("Updating keyboard");

                self.keyboard.update(event);
            },
            WindowEvent::Scroll(..)
            | WindowEvent::CursorPos(..)
            | WindowEvent::MouseButton(..)
            | WindowEvent::CursorEnter(_)
            | WindowEvent::Focus(_) => {
                #[cfg(feature = "trace")]
                debug!("Updating mouse");

                self.mouse.update(event);
            },
            _ => {
                /* Ignore any non-mouse, non-key events */
                #[cfg(feature = "trace")]
                warn!("Match defaulted. No update occurred.");
            }
        }

        // Remove newly released keys from "held collections"

        // Retain all buttons in the held_buttons hash map that have NOT been
        // released.
        let mouse_buttons = self.mouse.get_released_buttons();
        self.held_buttons.retain(|k, _| {
            !mouse_buttons.contains_key(k)
        });
        #[cfg(feature = "trace")]
        debug!("Updated held buttons.");

        // Same idea but using a difference function for a set.
        // held_keys = held_keys - released_keys
        self.held_keys = self.held_keys.difference(self.keyboard.get_released_keys()).copied().collect();
        #[cfg(feature = "trace")]
        debug!("Updated held keys.");
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn clear(&mut self) {
        // Add newly pressed keys to "held collections"
        self.held_buttons.extend(self.mouse.get_clicked_buttons());
        self.held_keys.extend(self.keyboard.get_pressed_keys());
        #[cfg(feature = "trace")]
        debug!("Updated held buttons to add buttons pressed down this cycle.");

        self.mouse.clear();
        self.keyboard.clear();
        #[cfg(feature = "trace")]
        debug!("Cleared mouse and keyboard.");
    }
}