use std::path::{Path, PathBuf};

use anyhow::Result;
use image::io::Reader;
use luminance::depth_test::DepthComparison;
use luminance_front::texture::{GenMipmaps, MagFilter, MinFilter, Sampler, Texture as LumTex, Wrap};
use luminance_front::pixel::Pixel;
use luminance_glfw::GL33Context;
use serde::Deserialize;
use specs::{Builder, Component, VecStorage, World};
use specs::storage::UnprotectedStorage;
use specs::world::LazyBuilder;
use thiserror::Error;

use crate::components::ComponentLoader;
use crate::globals::texture_dict::{TextureDict};
use crate::graphics::texture::TextureError::{FileNameDNE, PathNotFile, PathStringConversion, ReaderFailedToOpen, TextureDictDNE, RGB8ConversionFailed};
use crate::graphics::texture::TextureLoaderError::{CanNotDeserialize, ContextMissing, TextureDidNotLoad};
use crate::load::{JSONLoad, load_deserializable_from_json, LoadError};
use crate::loading::DrawTask;
use std::sync::{Arc, Mutex, RwLock};
use std::borrow::BorrowMut;
use std::ops::DerefMut;

#[derive(Debug, Clone)]
pub struct Texture {
    pub(crate) handle: String
}

impl Component for Texture { type Storage = VecStorage<Self>; }

impl Texture {
    const SAMPLER: Sampler = Sampler {
        wrap_r: Wrap::ClampToEdge,
        wrap_s: Wrap::ClampToEdge,
        wrap_t: Wrap::ClampToEdge,
        min_filter: MinFilter::Nearest,
        mag_filter: MagFilter::Nearest,
        depth_comparison: Some(DepthComparison::Less)
    };
    
    pub fn load(path: PathBuf, name: Option<String>) -> DrawTask<Self> {
        DrawTask::new(|(world, context)| {
            let path_string = path
                .to_str()
                .ok_or(
                    PathStringConversion {
                        path: path.clone()
                    }
                )?
                .to_string();

            if !path.is_file() { return Err(anyhow::Error::new(PathNotFile { path: path_string })) }

            let name = if let Some(name) = name {
                name
            } else {
                path.file_stem()
                    .ok_or(FileNameDNE {path: path_string.clone()})?
                    .to_string_lossy()
                    .to_string()
            };

            // eprintln!("Texture name={:?}", name.clone());
            // let mut ecs = world.lock().expect("Failed to lock World");
            // eprintln!("Acquired ecs");

            let world = world.read()
                .expect("Failed to lock World");

            let mut texture_dict = world.fetch_mut::<TextureDict>();
            eprintln!("Acquired texture_dict");

            let texture_handle = Self { handle: name.clone() };

            if !texture_dict.contains_key(&texture_handle) {
                let dynamic_image = Reader::open(path)
                    .map_err(|e| ReaderFailedToOpen {
                        path: path_string.clone(),
                        source: e
                    }
                    )?
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

                let mut ctx = context.write().expect("Failed to lock context");

                let texture = LumTex::new_raw(
                    ctx.deref_mut(),
                    [x, y],
                    0,
                    Self::SAMPLER,
                    GenMipmaps::No,
                    &rgb_image_rev
                )?;

                texture_dict.insert(&texture_handle, texture);
            }

            Ok(texture_handle)
        })
    }
}

#[derive(Error, Debug)]
pub enum TextureError {
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
    }
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
    fn from_json(json: JSONLoad) -> Result<Self> where Self: Sized {
        let json = load_deserializable_from_json(&json, &TEXTURE_LOAD_ID)
            .map_err(|e| { CanNotDeserialize {json, source: e} })?;

        Ok(Self{ json })
    }

    fn load_component<'a>(&self, builder: LazyBuilder<'a>, ecs: Arc<RwLock<World>>, context: Option<Arc<RwLock<GL33Context>>>) -> Result<LazyBuilder<'a>> {
        if let Some(context) = context {
            let texture = Texture::load(PathBuf::from(&self.json.image_path), self.json.name.clone())
                .execute((ecs, context))
                .map_err(|e| {
                    TextureDidNotLoad {
                        texture_info: self.json.clone(),
                        source: e
                    }
                })?;

            Ok(builder.with(texture))
        } else {
            Err(anyhow::Error::new(
                ContextMissing {
                    texture: self.json.clone()
                }
            ))
        }
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        self.json = load_deserializable_from_json(&new_value, &TEXTURE_LOAD_ID)
            .map_err(|e| { CanNotDeserialize {json: new_value, source: e} })?;

        Ok(())
    }

    fn get_component_name(&self) -> String {
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
    }
}
