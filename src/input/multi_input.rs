use crate::input::mouse::{Mouse, Button, CursorPosition, WheelMovement};
use crate::input::keyboard::{KeyBoard, Key};
use std::collections::{HashMap, HashSet};
use crate::input::Input;
use glfw::WindowEvent;

pub struct MultiInput {
    mouse: Mouse,
    keyboard: KeyBoard,
    held_buttons: HashMap<Button, CursorPosition>,
    held_keys: HashSet<Key>
}

impl MultiInput {
    pub fn get_cursor_position(&self) -> CursorPosition {
        return self.mouse.get_cursor_position()
    }

    pub fn get_wheel_movement(&self) -> WheelMovement {
        return self.mouse.get_wheel_movement()
    }

    pub fn is_cursor_owned(&self) -> bool {
        return self.mouse.is_cursor_owned()
    }

    pub fn is_cursor_within_window(&self) -> bool {
        return self.mouse.is_cursor_within_window()
    }

    pub fn get_clicked_buttons(&self) -> &HashMap<Button, CursorPosition> {
        self.mouse.get_clicked_buttons()
    }

    pub fn get_held_buttons(&self) -> &HashMap<Button, CursorPosition> {
        &self.held_buttons
    }

    pub fn get_released_buttons(&self) -> &HashMap<Button, CursorPosition>  {
        self.mouse.get_released_buttons()
    }

    pub fn get_pressed_keys(&self) -> &HashSet<Key> {
        self.keyboard.get_pressed_keys()
    }

    pub fn get_held_keys(&self) -> &HashSet<Key> {
        &self.held_keys
    }

    pub fn get_released_keys(&self) -> &HashSet<Key> {
        self.keyboard.get_released_keys()
    }
}

impl Input for MultiInput {
    fn new() -> Self {
        MultiInput {
            mouse: Mouse::new(),
            keyboard: KeyBoard::new(),
            held_buttons: HashMap::new(),
            held_keys: HashSet::new()
        }
    }

    fn update(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Key(..)
            | WindowEvent::Char(_)
            | WindowEvent::CharModifiers(..) => {
                self.keyboard.update(event);
            },
            WindowEvent::Scroll(..)
            | WindowEvent::CursorPos(..)
            | WindowEvent::MouseButton(..)
            | WindowEvent::CursorEnter(_)
            | WindowEvent::Focus(_) => {
                self.mouse.update(event);
            },
            _ => {/* Ignore any non-mouse, non-key events */}
        }

        // Remove newly released keys from "held collections"

        // Retain all buttons in the held_buttons hash map that have NOT been
        // released.
        let mouse_buttons = self.mouse.get_released_buttons();
        self.held_buttons.retain(|k, _| {
            !mouse_buttons.contains_key(k)
        });

        // Same idea but using a difference function for a set.
        // held_keys = held_keys - released_keys
        self.held_keys = self.held_keys.difference(self.keyboard.get_released_keys()).copied().collect();
    }

    fn clear(&mut self) {
        // Add newly pressed keys to "held collections"
        self.held_buttons = self.mouse.get_clicked_buttons().clone();
        self.held_keys = self.keyboard.get_pressed_keys().clone();

        self.mouse.clear();
        self.keyboard.clear();
    }
}