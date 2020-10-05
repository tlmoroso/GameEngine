use crate::components::{player_control::PlayerControl, position::Position};

use specs::prelude::*;

use coffee::input::keyboard::KeyCode;

use std::collections::HashSet;

const MOVE_UNIT: u16 = 32;

pub struct MovePlayer {
    pub keys_pressed: HashSet<KeyCode>,
}

impl<'a> System<'a> for MovePlayer {
    type SystemData = (
        ReadStorage<'a, PlayerControl>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (p, mut pos): Self::SystemData) {
        for (_, pos) in (&p, &mut pos).join() {
            if self.keys_pressed.contains(&KeyCode::Left) {
                pos.x = match pos.x {
                    n if n < MOVE_UNIT => pos.x,
                    _ => pos.x - MOVE_UNIT,
                };
            }
            
            if self.keys_pressed.contains(&KeyCode::Right) {
                pos.x = match pos.x {
                    n if n > u16::MAX - MOVE_UNIT => pos.x,
                    _ => pos.x + MOVE_UNIT,
                };
            }

            if self.keys_pressed.contains(&KeyCode::Up) {
                pos.y = match pos.y {
                    n if n < MOVE_UNIT => pos.y,
                    _ => pos.y - MOVE_UNIT,
                };
            }

            if self.keys_pressed.contains(&KeyCode::Down) {
                pos.y = match pos.y {
                    n if n > u16::MAX - MOVE_UNIT => pos.y,
                    _ => pos.y + MOVE_UNIT,
                };
            }
        }
    }
}
