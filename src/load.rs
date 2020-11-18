use serde_json::{Value, from_str, from_value};
use serde::Deserialize;

use std::fs::read_to_string;
use std::error::Error;

pub const LOAD_PATH: &str = "assets/JSON/";
pub const JSON_FILE: &str = ".json";

#[derive(Deserialize, Debug)]
pub struct JSONLoad {
    pub load_type_id: String,
    pub actual_value: Value
}

pub fn load_json(file_path: &String) -> Result<JSONLoad, Box<dyn Error>> {
    let json_string = read_to_string(file_path)?;
    let json_value = from_str::<Value>(json_string.as_str())?;
    from_value(json_value).map_err(|err| {Box::new(err) as Box<dyn Error>})
}