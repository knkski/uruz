use crate::api::v1 as apiv1;
use crate::clouds::Cloud;
use crate::rune::v1::rune::Rune;
use crate::server::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_slice;
use std::collections::VecDeque;
use uuid::Uuid;

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

impl Into<apiv1::Action> for Action {
    fn into(self) -> apiv1::Action {
        match self {
            Action::CreateModel { name } => apiv1::Action::CreateModel { name },
            Action::ConfigureModel { foo } => apiv1::Action::ConfigureModel { foo },
            Action::DestroyModel => apiv1::Action::DestroyModel,
            Action::AddRune { name, rune } => apiv1::Action::AddRune {
                name,
                rune: rune.into(),
            },
            Action::ConfigureRune {
                name,
                attribute,
                value,
            } => apiv1::Action::ConfigureRune {
                name,
                attribute,
                value,
            },
            Action::RemoveRune { name } => apiv1::Action::RemoveRune { name },
        }
    }
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

    pub fn get_status(&self) -> ModelStatus {
        let mut status = ModelStatus::Ready;

        for item in &self.history {
            match item.action {
                Action::CreateModel { .. } => status = ModelStatus::Ready,
                Action::DestroyModel => status = ModelStatus::Destroyed,
                _ => {}
            }
        }

        status
    }
}

impl Into<apiv1::Model> for Model {
    fn into(self) -> apiv1::Model {
        let mut state: apiv1::ModelState = Default::default();

        for item in &self.history {
            match &item.action {
                Action::CreateModel { .. } => state.status = apiv1::ModelStatus::Ready,
                Action::ConfigureModel { foo } => state.config.foo = foo.clone(),
                Action::DestroyModel => state.status = apiv1::ModelStatus::Destroyed,
                Action::AddRune { name, rune } => {
                    state.runes.insert(name.clone(), rune.clone().into());
                }
                Action::ConfigureRune {
                    name,
                    attribute,
                    value,
                } => {
                    let rune = state.runes.get_mut(name).unwrap();
                    rune.state.insert(attribute.clone(), Some(value.clone()));
                }
                Action::RemoveRune { name } => {
                    state.runes.remove(name).unwrap();
                }
            }
        }
        let mut requests: Vec<_> = self
            .history
            .into_iter()
            .map(|h| apiv1::Request {
                id: h.id,
                action: h.action.into(),
                queued: h.queued,
                started: Some(h.started),
                completed: Some(h.completed),
            })
            .collect();
        if let Some(a) = self.active {
            requests.push(apiv1::Request {
                id: a.id,
                action: a.action.into(),
                queued: a.queued,
                started: Some(a.started),
                completed: None,
            });
        }
        requests.extend(self.backlog.into_iter().map(|h| apiv1::Request {
            id: h.id,
            action: h.action.into(),
            queued: h.queued,
            started: None,
            completed: None,
        }));
        apiv1::Model {
            id: self.id.to_string(),
            name: self.name,
            cloud: self.cloud.to_string(),
            requests,
            state,
        }
    }
}
