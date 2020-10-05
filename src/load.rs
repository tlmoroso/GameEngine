use specs::{Component};
use serde_json::{Value, from_value, from_str};
use serde::Deserialize;
use std::fs::read_to_string;

use crate::scenes::{Scene};

pub const LOAD_PATH: &str = "assets/JSON/";
pub const JSON_FILE: &str = ".json";

#[derive(Deserialize, Debug)]
pub struct LoadJSON {
    pub loadable_type: String,
    pub other_value: Value,
}

pub trait Loadable: Sync + Send {}

pub trait ComponentLoadable: Loadable + Component {}
pub trait SceneLoadable: Loadable + Scene {}

pub fn load_json(file_path: String) -> LoadJSON {
    let json_string = read_to_string(file_path.clone())
        .expect(format!("ERROR: could not load data from file_path in Load::load: {}", file_path).as_str());
    let json_value = from_str::<Value>(json_string.as_str())
        .expect(format!("ERROR: could not parse json string into value in Load::load: {}", json_string).as_str());
    from_value(json_value)
        .expect("ERROR: could not translate Value into LoadJSON struct")
}