use glam::{Vec3, Mat4};
use crate::camera::Camera;
use std::sync::{RwLock, Arc, PoisonError, RwLockWriteGuard};
use thiserror::Error;
use serde::Deserialize;

#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};

use crate::loading::{DrawTask, Task};
use crate::load::{load_deserializable_from_file, LoadError};
use crate::camera::perspective_camera::PerspectiveCameraErrors::DeserializeError;

#[derive(Debug, Clone)]
pub struct PerspectiveCamera(Arc<RwLock<CameraValues>>);

#[derive(Debug, Copy, Clone)]
struct CameraValues {
    position: Vec3,
    target: Vec3,
    up_vec: Vec3,
    view: Mat4,
    change_flag: bool
}

impl Default for CameraValues {
    fn default() -> Self {
        CameraValues {
            position: Vec3::ZERO,
            target: Vec3::ZERO,
            up_vec: Vec3::Y,
            view: Mat4::ZERO,
            change_flag: false
        }
    }
}

impl Camera for PerspectiveCamera {

    #[cfg_attr(feature = "trace", instrument)]
    fn view(&mut self) -> Mat4 {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");

        if vars.change_flag {
            #[cfg(feature = "trace")]
            debug!("Change flag is set. Recalculating view matrix.");

            vars.view = Mat4::look_at_rh(
                vars.position,
                vars.target,
                vars.up_vec
            );
            vars.change_flag = false;
        }

        vars.view
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn position(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.position
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn set_position(&mut self, new_pos: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.position = new_pos;
        vars.change_flag = true;
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn translate_position(&mut self, translation: Mat4) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.position = translation.project_point3(vars.position);
        vars.change_flag = true;
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn target(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.target
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn set_target(&mut self, new_target: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.target = new_target;
        vars.change_flag = true;
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn translate_target(&mut self, translation: Mat4) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.target = translation.project_point3(vars.target);
        vars.change_flag = true;
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn up_vector(&self) -> Vec3 {
        let vars = self.0.read()
            .expect("Failed to acquire write lock for camera");
        vars.up_vec
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn set_up_vector(&mut self, new_vec: Vec3) {
        let mut vars = self.0.write()
            .expect("Failed to acquire write lock for camera");
        vars.up_vec = new_vec;
        vars.change_flag = true;
    }
}

pub const PERSPECTIVE_CAMERA_LOAD_ID: &str = "perspective_camera";


#[derive(Deserialize, Debug, Clone)]
pub struct PerspectiveCameraJSON {
    #[serde(default)]
    position: Option<[f32; 3]>,
    #[serde(default)]
    target: Option<[f32; 3]>,
    #[serde(default)]
    up_vec: Option<[f32; 3]>
}

#[derive(Debug, Clone)]
pub struct PerspectiveCameraLoader {
    path: String,
}

impl PerspectiveCameraLoader {
    #[cfg_attr(feature = "trace", instrument)]
    pub fn new(file_path: String) -> Self {
        Self {
            path: file_path
        }
    }

    #[cfg_attr(feature = "trace", instrument)]
    pub fn load(&self) -> DrawTask<PerspectiveCamera> {
        let path = self.path.clone();
        Task::new(move |_| {
            let json: PerspectiveCameraJSON = load_deserializable_from_file(&path, PERSPECTIVE_CAMERA_LOAD_ID)
                .map_err(|e| {
                    DeserializeError {
                        path: path.clone(),
                        source: e
                    }
                })?;

            Ok(PerspectiveCamera {
                0: Arc::new(RwLock::new(
                    CameraValues {
                        position: {
                            if let Some(position) = json.position {
                                Vec3::new(
                                    position[0],
                                    position[1],
                                    position[2]
                                )
                            } else {
                                Default::default()
                            }
                        } ,
                        target: if let Some(target) = json.target {
                            Vec3::new(target[0], target[1], target[2])
                        } else {
                            Default::default()
                        },
                        up_vec: if let Some (up) = json.up_vec {
                            Vec3::new(up[0], up[1], up[2])
                        } else {
                            Default::default()
                        },
                        ..Default::default()
                    }
                ))
            })
        })
    }
}

#[derive(Error, Debug)]
pub enum PerspectiveCameraErrors {
    #[error("Failed to load Perspective Camera JSON from file: {path:?}")]
    DeserializeError {
        path: String,
        source: LoadError
    },
}