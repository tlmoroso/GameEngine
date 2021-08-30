use glam::{Vec3, Mat4};
use crate::camera::Camera;
use std::sync::{RwLock, Arc, PoisonError, RwLockWriteGuard};
use thiserror::Error;

#[cfg(feature = "trace")]
use tracing::{debug, error};

pub struct TopDown2D(Arc<RwLock<TopDown2DValues>>);

struct TopDown2DValues {
    position: Vec3,
    target: Vec3,
    up_vec: Vec3,
    view: Mat4,
    change_flag: bool
}

impl Camera for TopDown2D {

    fn view(&mut self) -> Mat4 {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");

        if vars.change_flag {
            vars.view = Mat4::look_at_rh(
                vars.position,
                vars.target,
                vars.up_vec
            );
        }

        vars.view
    }

    fn position(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.position
    }

    fn set_position(&mut self, new_pos: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.position = new_pos;
    }

    fn translate_position(&mut self, translation: Mat4) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.position = translation.transform_point3(vars.position);
    }

    fn target(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.target
    }

    fn set_target(&mut self, new_target: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.target = new_target;
    }

    fn translate_target(&mut self, translation: Mat4) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.target = translation.transform_point3(vars.target);
    }

    fn up_vector(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.up_vec
    }

    fn set_up_vector(&mut self, new_vec: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.up_vec = new_vec;
    }
}