#[allow(non_snake_case)]
pub mod orthographic_camera;
pub mod perspective_camera;

use glam::{Mat4, Vec3};

pub trait Camera: Send + Sync {
    fn view(&self) -> Mat4;

    fn position(&self) -> Vec3;

    fn set_position(&self, new_pos: Vec3);

    fn translate_position(&self, translation: Mat4);

    fn target(&self) -> Vec3;

    fn set_target(&self, new_target: Vec3);

    fn translate_target(&self, translation: Mat4);

    fn up_vector(&self) -> Vec3;

    fn set_up_vector(&self, new_vec: Vec3);
}