use crate::components::{ComponentMux, ComponentLoader};
use crate::load::JSONLoad;

use anyhow::Result;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::Deserialize;
use serde_json::{Value, from_value};

use coffee::graphics::Window;

use specs::{EntityBuilder, World, Builder, Component, VecStorage};

use thiserror::Error;
use crate::test_globals::TestGlobalError::{LoadIDMatchError, TestComponentLoadIDError};

pub(crate) const TEST_COMPONENT_LOAD_ID: &str = "test_component";

pub struct TestComponentMux {}

impl ComponentMux for TestComponentMux {
    fn map_json_to_loader(json: JSONLoad) -> Result<Box<dyn ComponentLoader>> {
        return match json.load_type_id.as_str() {
            TEST_COMPONENT_LOAD_ID => Ok(Box::new(TestComponentLoader::new(json)?)),
            _ => Err(anyhow::Error::new(
                LoadIDMatchError {
                    load_type_id: json.load_type_id
                }
            ))
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TestComponent {
    pub number: u32,
    pub boolean: bool,
    pub text: String,
    pub array: Vec<u32>,
    pub map: HashMap<String, u32>
}

impl Component for TestComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct TestComponentLoader {
    pub cached_value: Value,
    component_name: String
}

impl TestComponentLoader {
    pub fn new(json: JSONLoad) -> Result<Self> {
        return if json.load_type_id == TEST_COMPONENT_LOAD_ID {
            Ok( Self {
                cached_value: json.actual_value,
                component_name: TEST_COMPONENT_LOAD_ID.to_string()
            })
        } else {
            Err(anyhow::Error::new(
                TestComponentLoadIDError {
                    expected_id: TEST_COMPONENT_LOAD_ID.to_string(),
                    actual_id: json.load_type_id
                })
            )
        }
    }
}

impl ComponentLoader for TestComponentLoader {
    fn load_component<'a>(&self, entity_task: EntityBuilder<'a>, ecs: Arc<RwLock<World>>, window: &Window) -> Result<EntityBuilder<'a>> {
        Ok(
            entity_task.with(
                from_value::<TestComponent>(self.cached_value.clone())
                    .map_err(|e| {
                        anyhow::Error::new(e)
                    })?
            )
        )
    }

    fn set_value(&mut self, new_value: JSONLoad) -> Result<()> {
        return if new_value.load_type_id == TEST_COMPONENT_LOAD_ID {
            self.cached_value = new_value.actual_value;
            Ok(())
        } else {
            Err(anyhow::Error::new(
                TestComponentLoadIDError {
                    expected_id: TEST_COMPONENT_LOAD_ID.to_string(),
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