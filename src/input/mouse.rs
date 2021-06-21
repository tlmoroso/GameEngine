use glfw::{MouseButton, Modifiers, WindowEvent, Action};
use std::collections::HashMap;
use crate::input::Input;

#[derive(Debug, Copy, Clone)]
pub struct WheelMovement {
    /// The number of horizontal lines scrolled
    pub horizontal: f64,

    /// The number of vertical lines scrolled
    pub vertical: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Button {
    pub button: MouseButton,
    pub modifiers: Modifiers
}

#[derive(Debug, Clone)]
pub struct Mouse {
    cursor_position: CursorPosition,
    wheel_movement: WheelMovement,
    is_cursor_owned: bool,
    is_cursor_within_window: bool,
    clicked_buttons: HashMap<Button, CursorPosition>,
    released_buttons: HashMap<Button, CursorPosition>
}

impl Mouse {
    pub fn get_cursor_position(&self) -> CursorPosition {
        return self.cursor_position
    }

    pub fn get_wheel_movement(&self) -> WheelMovement {
        return self.wheel_movement
    }

    pub fn is_cursor_owned(&self) -> bool {
        return self.is_cursor_owned
    }

    pub fn is_cursor_within_window(&self) -> bool {
        return self.is_cursor_within_window
    }

    pub fn get_clicked_buttons(&self) -> &HashMap<Button, CursorPosition> {
        &self.clicked_buttons
    }

    pub fn get_released_buttons(&self) -> &HashMap<Button, CursorPosition>  {
        &self.released_buttons
    }
}

impl Input for Mouse {
    fn new() -> Self {
        Mouse {
            cursor_position: CursorPosition { x: 0.0, y: 0.0 },
            wheel_movement: WheelMovement { horizontal: 0.0, vertical: 0.0 },
            is_cursor_owned: false,
            is_cursor_within_window: false,
            clicked_buttons: HashMap::new(),
            released_buttons: HashMap::new()
        }
    }

    fn update(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Focus(isFocused) => self.is_cursor_owned = isFocused,
            WindowEvent::CursorEnter(entered) => self.is_cursor_within_window = entered,
            WindowEvent::MouseButton(button, action, modifiers) => {
                match action {
                    Action::Release => {
                        self.released_buttons.insert(
                            Button {
                                button,
                                modifiers
                            },
                            self.cursor_position
                        );
                    }
                    Action::Press => {
                        self.clicked_buttons.insert(
                            Button {
                                button,
                                modifiers
                            },
                            self.cursor_position
                        );
                    }
                    Action::Repeat => {/*
                        Currently uninterested in this action.
                        NOT to be mistaken with isKeyHeldDown
                     */}
                }
            },
            WindowEvent::CursorPos(x, y) => {
                self.cursor_position = CursorPosition { x, y };
            },
            WindowEvent::Scroll( delta_x, delta_y) => {
                self.wheel_movement = WheelMovement {
                    horizontal: delta_x,
                    vertical: delta_y
                };
            },
            _ => {/* Ignore anything unrelated to the mouse */ }
        }
    }

    fn clear(&mut self) {
        self.clicked_buttons.clear();
        self.released_buttons.clear();
    }
}