use anyhow::Result;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Deserialize;
use serde_json::{Value, from_value};

use coffee::graphics::Window;

use specs::{EntityBuilder, World, Builder, Component, VecStorage};

use thiserror::Error;

use crate::test_globals::TestGlobalError::{LoadIDMatchError, TestComponentLoadIDError};

use game_engine::components::{ComponentMux, ComponentLoader};
use game_engine::load::JSONLoad;
use serde::de::DeserializeOwned;

pub const BASIC_TEST_NUMBER_COMPONENT_LOAD_ID: &str = "basic_test_number_component";
pub const BASIC_TEST_BOOLEAN_COMPONENT_LOAD_ID: &str = "basic_test_boolean_component";
pub const BASIC_TEST_TEXT_COMPONENT_LOAD_ID: &str = "basic_test_text_component";
pub const BASIC_TEST_VECTOR_COMPONENT_LOAD_ID: &str = "basic_test_vector_component";
pub const BASIC_TEST_MAP_COMPONENT_LOAD_ID: &str = "basic_test_map_component";

pub const TEST_LOAD_PATH: &str = "test_files/";

pub struct TestComponentMux {}

impl ComponentMux for TestComponentMux {
    fn map_json_to_loader(json: JSONLoad) -> Result<Box<dyn ComponentLoader>> {
        return match json.load_type_id.as_str() {
            BASIC_TEST_NUMBER_COMPONENT_LOAD_ID => Ok(Box::new(BasicTestComponentLoader::<BasicNumberTestComponent>::new(json)?)),
            _ => Err(anyhow::Error::new(
                LoadIDMatchError {
                    load_type_id: json.load_type_id
                }
            ))
        }
    }
}

pub trait BasicTestComponent: Component + DeserializeOwned + Debug + Send + Sync {
    const LOAD_ID: &'static str;
}

#[derive(Deserialize, Debug)]
pub struct BasicNumberTestComponent {
    pub number: u32
}

impl Component for BasicNumberTestComponent {
    type Storage = VecStorage<Self>;
}

impl BasicTestComponent for BasicNumberTestComponent {
    const LOAD_ID: &'static str = BASIC_TEST_NUMBER_COMPONENT_LOAD_ID;
}

#[derive(Debug)]
pub struct BasicTestComponentLoader<T: BasicTestComponent> {
    cached_value: Value,
    component_name: String,
    phantom: PhantomData<T>
}

// #[derive(Deserialize, Debug)]
// pub struct TestComponent {
//     pub number: u32,
//     pub boolean: bool,
//     pub text: String,
//     pub array: Vec<u32>,
//     pub map: HashMap<String, u32>
// }

impl<T: BasicTestComponent> BasicTestComponentLoader<T> {
    fn new(json: JSONLoad) -> Result<Self> {
        return if json.load_type_id == T::LOAD_ID {
            Ok( Self {
                cached_value: json.actual_value,
                component_name: T::LOAD_ID.to_string(),
                phantom: PhantomData
            })
        } else {
            Err(anyhow::Error::new(
                TestComponentLoadIDError {
                    expected_id: T::LOAD_ID.to_string(),
                    actual_id: json.load_type_id
                })
            )
        }
    }
}

impl<T: BasicTestComponent> ComponentLoader for BasicTestComponentLoader<T> {
    fn load_component<'b>(&self, entity_task: EntityBuilder<'b>, ecs: Arc<RwLock<World>>, window: &Window) -> Result<EntityBuilder<'b>> {
        Ok(
            entity_task.with(
                from_value::<T>(self.cached_value.clone())
                    .map_err(|e| {
                        anyhow::Error::new(e)
                    })?
            )
        )
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        return if new_value.load_type_id == T::LOAD_ID {
            self.cached_value = new_value.actual_value;
            Ok(())
        } else {
            Err(anyhow::Error::new(
                TestComponentLoadIDError {
                    expected_id: T::LOAD_ID.to_string(),
                    actual_id:  new_value.load_type_id
                }
            ))
        }
    }

    fn get_component_name(&self) -> String {
        return self.component_name.clone()
    }
}

#[derive(Error, Debug)]
pub enum TestGlobalError {
    #[error("Failed to match load_type_id: {load_type_id} to any component")]
    LoadIDMatchError {
        load_type_id: String
    },
    #[error("Given JSONLoad's load id: {actual_id} did not match TEST_COMPONENT_LOAD_ID: {expected_id}")]
    TestComponentLoadIDError {
        expected_id: String,
        actual_id: String
    }
}