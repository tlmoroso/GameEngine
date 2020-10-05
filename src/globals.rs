use std::collections::{HashMap};
use coffee::graphics::{Image, Font, Window};
use crate::load::Loadable;
use specs::World;
use coffee::load::Task;
use std::fs::{read};
use std::sync::{RwLock, Arc};
use serde_json::{Value, from_value};
use serde::Deserialize;

pub const FONT_DICT_FILE_ID: &str = "font_dict";

#[derive(Deserialize, Debug)]
struct FontMapJSON(HashMap<String, String>);

pub struct ImageDict(pub(crate) HashMap<String, Image>);
pub struct FontDict(pub(crate) Arc<RwLock<HashMap<String, Font>>>);

pub const FONTS_FILE_ID: &str = "fonts";

pub const TITLE_FONT: &str = "title_font";
static mut TITLE_FONT_DATA: Vec<u8> = Vec::new();

unsafe impl Send for FontDict {}

unsafe impl Sync for FontDict {}

impl Loadable for FontDict {}

impl FontDict {
    pub fn load(_ecs: Arc<RwLock<World>>, window: Arc<RwLock<&mut Window>>, json_value: Value) -> Task<Self> {
        let mut window_mut = window
            .write()
            .expect("ERROR: RwLock poisoned for window in FontDict::load");

        let font_map_json: FontMapJSON = from_value(json_value)
            .expect("ERROR: could not translate json value into font_map in FontDict::load");
        let mut font_dict = HashMap::new();
        for (font_name, font_path) in font_map_json.0 {
            let mut loaded_font: Font;
            unsafe {
                loaded_font = match font_name.as_str() {
                    TITLE_FONT => {
                        TITLE_FONT_DATA = read(font_path.clone())
                            .expect(format!("ERROR: could not load file into string in FontDict::load: font_path = {}", font_path).as_str());
                        Font::from_bytes(window_mut.gpu(), TITLE_FONT_DATA.as_slice())
                            .expect("ERROR: could not load font from font data in FontDict::load")
                    },
                    _ => panic!(format!("ERROR: font name did not match any fonts: {}", font_name))
                }
            }

            match font_name.as_str() {
                TITLE_FONT => font_dict.insert(TITLE_FONT.to_string(), loaded_font),
                _ => panic!(format!("ERROR: font name did not match any valid names: {}", font_name)),
            };
        }

        Task::new( || {
            Ok(
                FontDict(
                    Arc::new(RwLock::new(font_dict))
                )
            )
        })
    }
}