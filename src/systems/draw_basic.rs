use coffee::graphics::{Frame, Color};
use specs::{System, Write, ReadStorage, Join, Read};
use crate::globals::{image_dict::ImageDict, font_dict::FontDict};
use crate::components::drawables::Drawable;
use std::borrow::BorrowMut;
use std::collections::HashSet;

pub struct DrawBasic<'a, 'b> {
    pub frame: &'a mut Frame<'b>,
}

impl<'a, 'b> System<'a> for DrawBasic<'a, 'b> {
    type SystemData = (
        Write<'a, FontDict>,
        Read<'a, ImageDict>,
        ReadStorage<'a, Drawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut font_dict, image_dict, drawables) = data;

        self.frame.clear(Color::BLACK);
        let mut target = self.frame.as_target();

        for drawable in (&drawables).join() {
            if let Some(shapes) = &drawable.shapes {
                for mesh in shapes {
                    mesh.mesh.draw(target.borrow_mut());
                }
            }

            if let Some(texts) = &drawable.text {
                for text in texts {
                    let mut font = font_dict.0.get_mut(text.font.as_str())
                        .expect(format!("Failed to get font: {:?} from FontDict in DrawTextBox system", text.font).as_str());

                    let coffee_text = text.into();

                    font.add(coffee_text);
                    font.draw(target.borrow_mut());
                }
            }

            if let Some(sprites) = &drawable.sprites {
                for sprite in sprites {
                    let image = image_dict.0.get(sprite.image.as_str())
                        .expect(format!("ERROR: Could not retrieve image: {:#?} from image_dict: {:#?}", sprite.image, image_dict.0).as_str());

                    image.draw(sprite.sprite.clone(), target.borrow_mut());
                }
            }

            if let Some(sprites) = &drawable.animated_sprites {
                for sprite in sprites {
                    let image = image_dict.0.get(sprite.image.as_str())
                        .expect(format!("ERROR: Could not retrieve image: {:#?} from image_dict: {:#?}", sprite.image, image_dict.0).as_str());

                    image.draw(sprite.sprite.clone(), target.borrow_mut());
                }
            }
        }
    }
}