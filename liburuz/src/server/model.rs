use crate::clouds::Cloud;
use crate::server::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_slice;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Action {
    CreateModel { name: String },
    ConfigureModel { foo: String },
    DestroyModel,
    AddRune,
    RemoveRune,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Queued {
    pub id: Uuid,
    pub action: Action,
    pub queued: u128,
}

impl Queued {
    pub fn from_action(action: Action, queued: u128) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            queued,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Active {
    pub id: Uuid,
    pub action: Action,
    pub queued: u128,
    pub started: u128,
}

impl Active {
    pub fn from_queued(queued: Queued, started: u128) -> Self {
        Self {
            id: queued.id,
            action: queued.action,
            queued: queued.queued,
            started,
        }
    }

    pub fn get_action(&self) -> &Action {
        &self.action
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Completed {
    pub id: Uuid,
    pub action: Action,
    pub queued: u128,
    pub started: u128,
    pub completed: u128,
}

impl Completed {
    pub fn from_active(active: Active, completed: u128) -> Self {
        Self {
            id: active.id,
            action: active.action,
            queued: active.queued,
            started: active.started,
            completed,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub foo: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self { foo: "bar".into() }
    }
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
    pub runes: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub cloud: Cloud,
    pub backlog: VecDeque<Queued>,
    pub active: Option<Active>,
    pub history: Vec<Completed>,
}

impl Model {
    pub fn with_name(name: String, cloud: Cloud) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            cloud,
            backlog: VecDeque::new(),
            active: None,
            history: Vec::new(),
        }
    }

    pub fn from_tree(tree: &sled::Tree) -> Result<Self, Error> {
        macro_rules! get {
            ($attr:literal) => {
                &tree
                    .get($attr)?
                    .ok_or_else(|| Error::ModelLoad(format!("Attribute {} not found", $attr)))?;
            };
        }
        Ok(Self {
            id: Uuid::from_slice(get!("id"))?,
            name: String::from_utf8_lossy(get!("name")).to_string(),
            cloud: from_slice(get!("cloud"))?,
            backlog: from_slice(get!("backlog"))?,
            active: from_slice(get!("active"))?,
            history: from_slice(get!("history"))?,
        })
    }

    pub fn get_state(&self) -> ModelState {
        let mut state: ModelState = Default::default();

        for item in &self.history {
            match &item.action {
                Action::CreateModel { name: _ } => state.status = ModelStatus::Ready,
                Action::ConfigureModel { foo } => state.config.foo = foo.clone(),
                Action::DestroyModel => state.status = ModelStatus::Destroyed,
                Action::AddRune => {
                    state.runes.insert("foo".into(), "bar".into());
                }
                Action::RemoveRune => {
                    state.runes.remove("foo").unwrap();
                }
            }
        }

        state
    }

    pub fn get_status(&self) -> ModelStatus {
        self.get_state().status
    }
}
