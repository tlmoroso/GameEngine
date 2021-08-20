#[cfg(feature="trace")]
use tracing::{instrument, trace, error, debug};

use std::collections::HashMap;

use crate::load::{LoadError, load_deserializable_from_file};

use serde::Deserialize;

use thiserror::Error;
use crate::loading::DrawTask;
use std::path::Path;
use luminance_front::pixel::{Pixel, RGBA8UI};
use luminance_front::texture::{Texture, Sampler, Wrap, MinFilter, MagFilter, GenMipmaps, Dim2};
use luminance_glfw::GL33Context;
use luminance_front::context::GraphicsContext;
use anyhow::Result;
use image::io::Reader;
use luminance_front::depth_test::DepthComparison;
use crate::graphics::texture::TextureHandle;
use crate::globals::texture_dict::TextureDictError::{PathConversionFailed, RGB8ConversionFailed, WorldWriteLockError, TextureDictFileLoadError};
use luminance::pixel::RGB8UI;
use specs::World;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use image::Pixels;

pub const TEXTURE_DICT_LOAD_ID: &str = "texture_dict";

#[derive(Default)]
pub struct TextureDict(HashMap<String, Texture<Dim2, RGBA8UI>>);

unsafe impl Send for TextureDict {}
unsafe impl Sync for TextureDict {}

pub const IMAGES_DIR: &str = "images/";

#[derive(Deserialize, Debug, Clone)]
pub struct TextureDictLoader {
    json: TextureDictJSON
}

#[derive(Deserialize, Debug, Clone)]
struct TextureDictJSON {
    textures: HashMap<String, String>
}

impl TextureDictLoader {
    const SAMPLER: Sampler = Sampler {
        wrap_r: Wrap::ClampToEdge,
        wrap_s: Wrap::ClampToEdge,
        wrap_t: Wrap::ClampToEdge,
        min_filter: MinFilter::Nearest,
        mag_filter: MagFilter::Nearest,
        depth_comparison: Some(DepthComparison::Less)
    };

    #[cfg_attr(feature="trace", instrument(skip(file_path)))]
    pub fn new(file_path: &impl AsRef<Path>) -> Result<Self> {
        // Accept anything that can be interpreted as a Path, then convert it to a String for easier use
        let path = file_path
            .as_ref()
            .to_str()
            .ok_or_else(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to convert file path into string.");

                PathConversionFailed
            })?
            .to_string(); // Maybe there's a better way to do this?

        let json = load_deserializable_from_file(&path, TEXTURE_DICT_LOAD_ID)
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to deserialize file: ({:?}) into TextureDict JSON value", path.clone());

                TextureDictFileLoadError {
                    path: path.clone(),
                    source: e
                }
            })?;

        let new = Self {
            json
        };

        #[cfg(feature = "trace")]
        debug!("Successfully created new TextureDictLoader: {:?}", new.clone());

        return Ok(new)
    }

    #[cfg_attr(feature="trace", instrument)]
    pub fn load(self) -> DrawTask<TextureDict> {
        DrawTask::new(move |(_, context)| {
            #[cfg(feature="trace")]
            trace!("ImageDictJSON: ({:#?}) successfully loaded from: {:#?}", self.json.clone(), path.clone());

            let mut texture_dict = HashMap::new();

            let mut ctx = context.write()
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for World");

                    WorldWriteLockError
                })?;

            for (image_name, image_path) in self.json.textures {
                #[cfg(feature="trace")]
                debug!("Adding {:#?} at {:#?} to new TextureDict", image_name.clone(), image_path.clone());

                let dynamic_image = Reader::open(image_path.clone())?
                    .decode()?;
                let rgb_image = dynamic_image
                    .into_rgba8();

                #[cfg(feature = "trace")]
                debug!("Loaded image from file: ({:?}). Converted to rgb_image", image_path.clone());

                let rgb_image_rev: Vec<u8> = rgb_image.rows()
                    // Reverse the contents of each row a.k.a mirror it
                    // and get rid of the Rev iter layer using flat_map instead of map
                    .flat_map(|row| {
                        row.rev()
                    })
                    // Reverse all the rows a.k.a flip upside down
                    .rev()
                    // Flat_map expects an iter as the return value and automatically flattens it
                    // so we can use it as another way to convert a vec of pixels into the raw bytes
                    .flat_map(|pixel| {
                        pixel.0
                    })
                    .collect();

                #[cfg(feature = "trace")]
                debug!("Image reversed for texture and converted into raw bytes.");

                let (x, y) = rgb_image.dimensions();
                #[cfg(feature = "trace")]
                debug!("Image dimensions: ({:?}, {:?})", x, y);

                let texture = Texture::new_raw(
                    ctx.deref_mut(),
                    [x, y],
                    0,
                    Self::SAMPLER,
                    GenMipmaps::Yes,
                    &rgb_image_rev
                ).map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to create texture from image. Name: ({:?}). Path: {:?}", image_name.clone(), image_path.clone());

                    return e
                })?;

                #[cfg(feature = "trace")]
                debug!("Texture created.");

                texture_dict.insert(image_name, texture);

                #[cfg(feature = "trace")]
                debug!("Texture inserted into texture_dict");
            }

            #[cfg(feature = "trace")]
            debug!("Loaded and returning TextureDict. Keys: {:?}", texture_dict.keys());

            return Ok(TextureDict(texture_dict))
        })
    }
}

impl TextureDict {
    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    pub fn contains_key(&self, key: &TextureHandle) -> bool {
        self.0.contains_key(&key.handle)
    }

    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    pub fn get(&self, key: &TextureHandle) -> Option<&Texture<Dim2, RGBA8UI>> {
        self.0.get(&key.handle)
    }

    #[cfg_attr(feature = "trace", instrument(skip(self)))]
    pub fn get_mut(&mut self, key: &TextureHandle) -> Option<&mut Texture<Dim2, RGBA8UI>> {
        self.0.get_mut(&key.handle)
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, value)))]
    pub fn insert(&mut self, key: &TextureHandle, value: Texture<Dim2,RGBA8UI>) -> Option<Texture<Dim2,RGBA8UI>> {
        self.0.insert(key.handle.clone(), value)
    }
}

#[derive(Error, Debug)]
pub enum TextureDictError {
    #[error("Error loading JSON Value for ImageDictLoader from: {path}")]
    TextureDictFileLoadError {
        path: String,
        source: LoadError
    },

    #[error("Could not convert Path ref to str")]
    PathConversionFailed,

    #[error("Failed to convert dynamic image into RGB8. path={image_path}, name={image_name}")]
    RGB8ConversionFailed {
        image_name: String,
        image_path: String
    },
    #[error("Failed to acquire write lock for World")]
    WorldWriteLockError
}