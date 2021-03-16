use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Rune {
    pub transformers: Option<String>,
    pub react: Option<String>,
    pub state: HashMap<String, Option<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Action {
    CreateModel {
        name: String,
    },
    ConfigureModel {
        foo: Option<String>,
    },
    DestroyModel,
    AddRune {
        name: String,
        rune: Rune,
    },
    ConfigureRune {
        name: String,
        attribute: String,
        value: String,
    },
    RemoveRune {
        name: String,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Request {
    pub id: Uuid,
    pub action: Action,
    pub queued: u128,
    pub started: Option<u128>,
    pub completed: Option<u128>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub foo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Requested,
    Creating,
    Ready,
    Configuring,
    Destroyed,
}

impl Default for ModelStatus {
    fn default() -> ModelStatus {
        ModelStatus::Requested
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelState {
    pub status: ModelStatus,
    pub config: ModelConfig,
    pub runes: HashMap<String, Rune>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub cloud: String,
    pub requests: Vec<Request>,
    pub state: ModelState,
}
