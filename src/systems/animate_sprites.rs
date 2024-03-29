use specs::{System, WriteStorage, Join};
use crate::components::drawables::Drawable;

pub struct AnimateSprites;

impl<'a> System<'a> for AnimateSprites {
    type SystemData = (
        WriteStorage<'a, Drawable>
    );

    fn run(&mut self, data: Self::SystemData) {
        let mut drawables = data;

        for mut drawable in (&mut drawables).join() {
            if let Some(sprites) = &mut drawable.animated_sprites {
                for sprite in sprites {
                    let total_frames = (sprite.end_frame - sprite.start_frame + 1) * sprite.frame_pause;

                    if sprite.frame_pause_counter == total_frames {
                        sprite.frame_pause_counter = 0;
                        let height_difference = (sprite.end_frame - sprite.start_frame) * sprite.sprite.source.height;
                        sprite.sprite.source.y = sprite.sprite.source.y - height_difference;
                    }

                    if sprite.frame_pause_counter != 0 && sprite.frame_pause_counter % sprite.frame_pause == 0 {
                        sprite.sprite.source.y += sprite.sprite.source.height;
                    }

                    sprite.frame_pause_counter += 1;
                }
            }
        }
    }
}