use specs::prelude::*;

use coffee::graphics::Frame;

use crate::components::animation::Animation;
use crate::components::position::Position;
use std::borrow::BorrowMut;

pub struct AnimationPlayer<'a, 'b> {
    pub frame: &'a mut Frame<'b>,
}

impl<'a> System<'a> for AnimationPlayer<'_, '_> {
    type SystemData = (
            WriteStorage<'a, Animation>,
            WriteStorage<'a, Position>,
        );

    fn run(&mut self, (mut animation, mut position): Self::SystemData) {
        for (an, pos) in (&mut animation, &mut position).join() {
            let sprite = an.create_sprite(pos);
            let graphic = an.image.clone();
            graphic.draw(sprite, self.frame.as_target().borrow_mut());
        }
    }
}
