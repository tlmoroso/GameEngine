#[cfg(feature="trace")]
use tracing::{instrument, trace, error};

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
use crate::graphics::texture::Texture as TextureHandle;
use crate::globals::texture_dict::TextureDictError::{PathConversionFailed, RGB8ConversionFailed};
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

#[derive(Deserialize, Debug)]
pub struct TextureDictLoader {
    path: String
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

    #[cfg_attr(feature="trace", instrument)]
    pub fn new(file_path: &impl AsRef<Path>) -> Result<Self> {
        #[cfg(feature="trace")]
        trace!("ENTER: ImageDictLoader::new");
        let new = Self {
            // Accept anything that can be interpreted as a Path, then convert it to a String for easier use
            path: file_path.as_ref().to_str().ok_or(PathConversionFailed)?.to_string() // Maybe there's a better way to do this?
        };

        #[cfg(feature="trace")]
        trace!("EXIT: ImageDictLoader::new");
        return Ok(new)
    }

    #[cfg_attr(feature="trace", instrument(skip(self, _ecs, _window)))]
    pub fn load(self) -> DrawTask<TextureDict> {
        #[cfg(feature="trace")]
        trace!("ENTER: TextureDictLoader::load");

        let path = self.path.clone();

        DrawTask::new(move |u| {
            let (_, context) = u;
            let texture_dict_json: TextureDictJSON = load_deserializable_from_file(path.as_str(), TEXTURE_DICT_LOAD_ID)?;

            #[cfg(feature="trace")]
            trace!("ImageDictJSON: {:#?} successfully loaded from: {:#?}", image_dict_json, path.clone());

            let mut texture_dict = HashMap::new();

            let mut ctx = context.write().expect("Failed to lock context");

            for (image_name, image_path) in texture_dict_json.textures {
                #[cfg(feature="trace")]
                trace!("Adding {:#?} at {:#?} to ImageDict", image_name.clone(), image_path.clone());
                eprintln!("Loading image: name={:?} path={:?}", image_name.clone(), image_path.clone());
                let dynamic_image = Reader::open(image_path.clone())?
                    .decode()?;
                let rgb_image = dynamic_image
                    .into_rgba8();

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

                let (x, y) = rgb_image.dimensions();

                let texture = Texture::new_raw(
                    ctx.deref_mut(),
                    [x, y],
                    0,
                    Self::SAMPLER,
                    GenMipmaps::Yes,
                    &rgb_image_rev
                )?;

                texture_dict.insert(image_name, texture);
            }

            return Ok(TextureDict(texture_dict))
        })
    }
}

impl TextureDict {
    pub fn contains_key(&self, key: &TextureHandle) -> bool {
        self.0.contains_key(&key.handle)
    }

    pub fn get(&self, key: &TextureHandle) -> Option<&Texture<Dim2, RGBA8UI>> {
        self.0.get(&key.handle)
    }

    pub fn get_mut(&mut self, key: &TextureHandle) -> Option<&mut Texture<Dim2, RGBA8UI>> {
        self.0.get_mut(&key.handle)
    }

    pub fn insert(&mut self, key: &TextureHandle, value: Texture<Dim2,RGBA8UI>) -> Option<Texture<Dim2,RGBA8UI>> {
        self.0.insert(key.handle.clone(), value)
    }
}

#[derive(Error, Debug)]
pub enum TextureDictError {
    #[error("Error loading JSON Value for ImageDictLoader from: {var_name} = {path}")]
    TextureDictFileLoadError {
        path: String,
        var_name: String,
        source: LoadError
    },

    #[error("Could not convert Path ref to str")]
    PathConversionFailed,

    #[error("Failed to convert dynamic image into RGB8. path={image_path}, name={image_name}")]
    RGB8ConversionFailed {
        image_name: String,
        image_path: String
    }
}