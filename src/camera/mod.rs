#[allow(non_snake_case)]
pub mod orthographic_camera;
pub mod perspective_camera;

use glam::{Mat4, Vec3};

pub trait Camera: Send + Sync {
    fn view(&mut self) -> Mat4;

    fn position(&self) -> Vec3;

    fn set_position(&mut self, new_pos: Vec3);

    fn translate_position(&mut self, translation: Mat4);

    fn target(&self) -> Vec3;

    fn set_target(&mut self, new_target: Vec3);

    fn translate_target(&mut self, translation: Mat4);

    fn up_vector(&self) -> Vec3;

    fn set_up_vector(&mut self, new_vec: Vec3);
}