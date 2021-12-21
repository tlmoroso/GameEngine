// pub mod Texture2D;

use std::borrow::BorrowMut;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use image::ImageError;
use image::io::Reader;
use luminance::depth_test::DepthComparison;
use luminance_front::pixel::Pixel;
use luminance_front::texture::{GenMipmaps, MagFilter, MinFilter, Sampler, Texture as LumTex, Wrap};
use luminance_glfw::GL33Context;
use serde::Deserialize;
use specs::{Builder, Component, VecStorage, World};
use specs::storage::UnprotectedStorage;
use specs::world::LazyBuilder;
use thiserror::Error;
#[cfg(feature = "trace")]
use tracing::{debug, error, instrument};

use crate::components::ComponentLoader;
use crate::globals::texture_dict::TextureDict;
use crate::graphics::texture::TextureLoaderError::{CanNotDeserialize, ContextMissing, ContextWriteLockError, DecodeError, FileNameDNE, PathNotFile, PathStringConversion, ReaderFailedToOpen, RGB8ConversionFailed, TextureDictDNE, TextureDidNotLoad, WorldReadLockError};
use crate::load::{JSONLoad, load_deserializable_from_json, LoadError};
use crate::loading::DrawTask;
use crate::graphics::Context;

#[derive(Debug, Clone)]
pub struct TextureHandle {
    pub(crate) handle: String,
}

impl Component for TextureHandle { type Storage = VecStorage<Self>; }

impl TextureHandle {
    const SAMPLER: Sampler = Sampler {
        wrap_r: Wrap::ClampToEdge,
        wrap_s: Wrap::ClampToEdge,
        wrap_t: Wrap::ClampToEdge,
        min_filter: MinFilter::Nearest,
        mag_filter: MagFilter::Nearest,
        depth_comparison: Some(DepthComparison::Less)
    };
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextureJSON {
    #[serde(default)]
    pub name: Option<String>,
    pub image_path: String
}

#[derive(Debug)]
pub struct TextureLoader {
    pub json: TextureJSON
}

pub const TEXTURE_LOAD_ID: &str = "texture";

impl ComponentLoader for TextureLoader {
    #[cfg_attr(feature = "trace", instrument)]
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let texture_json: TextureJSON = load_deserializable_from_json(&json, &TEXTURE_LOAD_ID)
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to deserialize JSONLoad value: ({:?}) into TextureJSON type", json.clone());

                CanNotDeserialize {
                    json: json.clone(),
                    source: e
                }
            })?;

        #[cfg(feature = "trace")]
        debug!("Converted JSONLoad value: ({:?}) into TextureJSON value: {:?}", json.clone(), texture_json.clone());

        Ok(Self{ json: texture_json })
    }

    #[cfg_attr(feature = "trace", instrument(skip(builder, ecs, context)))]
    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: Arc<RwLock<World>>) -> Result<LazyBuilder<'a>> {
        let path = PathBuf::from(self.json.image_path.clone());

        if !path.is_file() {
            #[cfg(feature = "trace")]
            error!("Given path: ({:?}) does not point to file", self.json.image_path.clone());

            return Err(anyhow::Error::new(PathNotFile { path: self.json.image_path.clone() }))
        }

        let name = if let Some(name) = self.json.name.clone() {
            #[cfg(feature = "trace")]
            debug!("Optional name was given for texture: {:?}", name.clone());

            name
        } else {
            let name = path.file_stem()
                .ok_or_else(|| {
                    #[cfg(feature = "trace")]
                    error!("Could not get file stem(a.k.a file name) of path: {:?}", self.json.image_path.clone());

                    FileNameDNE {
                        path: self.json.image_path.clone()
                    }
                })?
                .to_string_lossy()
                .to_string();

            #[cfg(feature = "trace")]
            debug!("Name not given. Converted file name into image name: {:?}", name.clone());

            name
        };

        let world = ecs.read()
            .map_err(|_| {
                #[cfg(feature = "trace")]
                error!("Failed to acquire read lock for world");

                WorldReadLockError
            })?;

        let mut texture_dict = world.fetch_mut::<TextureDict>();
        #[cfg(feature = "trace")]
        debug!("Fetched texture store from ECS.");

        let texture_handle = TextureHandle { handle: name.clone() };

        if !texture_dict.contains_key(&texture_handle) {
            #[cfg(feature = "trace")]
            debug!("This is a new texture. It needs to be loaded from file and stored in the Texture Store.");

            let dynamic_image = Reader::open(path)
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to open image file at path: {:?}", self.json.image_path.clone());

                    ReaderFailedToOpen {
                        path: self.json.image_path.clone(),
                        source: e
                    }
                })?
                .decode()
                .map_err(|e| {
                    #[cfg(feature = "trace")]
                    error!("Failed to decode image at path: {:?}", self.json.image_path.clone());

                    DecodeError {
                        source: e,
                        image_path: self.json.image_path.clone()
                    }
                })?;

            let rgb_image = dynamic_image
                .into_rgba8();

            #[cfg(feature = "trace")]
            debug!("Successfully converted image from file into RGBA8 format");

            let rgb_image_rev: Vec<u8> = rgb_image.rows()
                // Reverse the contents of each row a.k.a mirror it
                // and get rid of the Rev iter layer using flat_map instead of map
                .flat_map(|row| {
                    row.rev()
                })
                // Reverse all rows a.k.a flip upside down
                .rev()
                // Flat_map expects an iter as the return value and automatically flattens it
                // so we can use it as another way to convert a vec of pixels into the raw bytes
                .flat_map(|pixel| {
                    pixel.0
                })
                .collect();
            #[cfg(feature = "trace")]
            debug!("Flipped and mirrored image so it is drawn properly by renderer.");

            let (x, y) = rgb_image.dimensions();
            #[cfg(feature = "trace")]
            debug!("Image size is x: {:?}, y: {:?}", x, y);

            let mut context = world.fetch_mut::<Context>();

            let mut ctx = context.0.write()
                .map_err(|_| {
                    #[cfg(feature = "trace")]
                    error!("Failed to acquire write lock for Context");

                    ContextWriteLockError
                })?;

            let texture = LumTex::new_raw(
                ctx.deref_mut(),
                [x, y],
                0,
                TextureHandle::SAMPLER,
                GenMipmaps::No,
                &rgb_image_rev
            )?;

            #[cfg(feature = "trace")]
            debug!("Created texture from raw image bytes. Storing in Texture Store.");

            texture_dict.insert(&texture_handle, texture);
        }

        #[cfg(feature = "trace")]
        debug!("Successfully created Texture. Adding to builder.");

        Ok(builder.with(texture_handle))
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        self.json = load_deserializable_from_json(&new_value, &TEXTURE_LOAD_ID)
            .map_err(|e| {
                #[cfg(feature = "trace")]
                error!("Failed to convert JSONLoad value: ({:?}) into TextureJSON", new_value.clone());

                CanNotDeserialize {
                    json: new_value.clone(),
                    source: e
                }
            })?;

        #[cfg(feature = "trace")]
        debug!("Successfully converted new JSONLoad value: ({:?}) into TextureJSON type: ({:?}) and replaced old value.", new_value.clone(), self.json.clone());

        Ok(())
    }

    #[cfg_attr(feature = "trace", instrument)]
    fn get_component_name(&self) -> String {
        #[cfg(feature = "trace")]
        debug!("Returning component name: {:?}", TEXTURE_LOAD_ID.to_string());

        return TEXTURE_LOAD_ID.to_string()
    }
}

#[derive(Error, Debug)]
pub enum TextureLoaderError {
    #[error("Failed to deserialize json from JSONLoad value={json:?}")]
    CanNotDeserialize {
        json: JSONLoad,
        source: LoadError
    },

    #[error("Context is required but value was None")]
    ContextMissing {
        texture: TextureJSON
    },

    #[error("Failed to load texture")]
    TextureDidNotLoad {
        texture_info: TextureJSON,
        source: anyhow::Error
    },
    #[error("TextureDict could not be retrieved from World")]
    TextureDictDNE,

    #[error("Could not open image file at {path}")]
    ReaderFailedToOpen {
        path: String,
        source: std::io::Error
    },

    #[error("Could not convert path={path} to String")]
    PathStringConversion {
        path: PathBuf,
    },

    #[error("Path={path} does not describe a file")]
    PathNotFile {
        path: String
    },

    #[error("File name could not be retrieved for path={path}")]
    FileNameDNE {
        path: String
    },

    #[error("Failed to convert dynamic image into RGB8. path={image_path}, name={image_name}")]
    RGB8ConversionFailed {
        image_path: String,
        image_name: String
    },

    #[error("Failed to acquire read lock for World")]
    WorldReadLockError,

    #[error("Failed to acquire write lock for Context")]
    ContextWriteLockError,

    #[error("Failed to decode image reader into DynamicImage from image at {image_path}")]
    DecodeError {
        source: ImageError,
        image_path: String
    }
}
