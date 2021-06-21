#[cfg(feature="trace")]
use tracing::{instrument, trace, error};

use std::collections::HashMap;
use std::io::ErrorKind;

use coffee::graphics::Image;

use crate::load::{LoadError, load_deserializable_from_file, build_task_error};
use self::ImageDictError::ImageDictFileLoadError;

use serde::Deserialize;

use thiserror::Error;
use crate::loading::Task;

pub const IMAGE_DICT_LOAD_ID: &str = "image_dict";

#[derive(Default, Debug)]
pub struct ImageDict(pub HashMap<String, Image>);

pub const IMAGES_DIR: &str = "images/";

#[derive(Deserialize, Debug)]
pub struct ImageDictLoader {
    path: String
}

#[derive(Deserialize, Debug, Clone)]
struct ImageDictJSON {
    images: HashMap<String, String>
}

impl ImageDictLoader {
    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: String) -> Self {
        #[cfg(feature="trace")]
        trace!("ENTER: ImageDictLoader::new");
        let new = Self {
            path: file_path
        };
        #[cfg(feature="trace")]
        trace!("EXIT: ImageDictLoader::new");
        return new
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _ecs, _window)))]
    pub fn load(self) -> Task<ImageDict> {
        #[cfg(feature="trace")]
        trace!("ENTER: ImageDictLoader::load");
        let mut image_task = Task::new(|| { Ok(
            HashMap::new()
        )});

        let image_dict_json: ImageDictJSON = map_err_return!(
            load_deserializable_from_file(self.path.as_str(), IMAGE_DICT_LOAD_ID),
            |e| { build_task_error(
                ImageDictFileLoadError {
                    path: self.path,
                    var_name: stringify!(self.path).to_string(),
                    source: e
                }
            )}
        );

        #[cfg(feature="trace")]
        trace!("ImageDictJSON: {:#?} successfully loaded from: {:#?}", image_dict_json, self.path);

        for (image_name, image_path) in image_dict_json.images {
            #[cfg(feature="trace")]
            trace!("Adding {:#?} at {:#?} to ImageDict", image_name.clone(), image_path.clone());

            // image_task.join()
            // image_task = (
            //     Image::load(image_path),
            //     image_task
            // )
            //     .join()
            //     .map(|(image, mut image_dict)| {
            //         image_dict.insert(image_name, image);
            //         return image_dict
            //     })
        }

        let task = image_task.map(|image_dict| {
            ImageDict(image_dict)
        });

        #[cfg(feature="trace")]
        trace!("EXIT: ImageDictLoader::load");
        return task
    }
}

#[derive(Error, Debug)]
pub enum ImageDictError {
    #[error("Error loading JSON Value for ImageDictLoader from: {var_name} = {path}")]
    ImageDictFileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    }
}