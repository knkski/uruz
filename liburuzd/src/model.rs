use crate::clouds::Cloud;
use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_slice;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    CreateModel { name: String },
    ConfigureModel { foo: String },
    DestroyModel,
    AddRune,
    RemoveRune,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Queued {
    pub id: Uuid,
    pub action: Action,
}

impl Queued {
    pub fn from_action(action: Action) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Active {
    id: Uuid,
    action: Action,
    started: u128,
}

impl Active {
    pub fn from_queued(queued: Queued, started: u128) -> Self {
        Self {
            id: queued.id,
            action: queued.action,
            started,
        }
    }

    pub fn get_action(&self) -> &Action {
        &self.action
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Completed {
    id: Uuid,
    action: Action,
    started: u128,
    completed: u128,
}

impl Completed {
    pub fn from_active(active: Active, completed: u128) -> Self {
        Self {
            id: active.id,
            action: active.action,
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

    pub fn from_tree(tree: sled::Tree) -> Result<Self, Error> {
        Ok(Self {
            id: Uuid::from_slice(&tree.get("id")?.unwrap()[..]).unwrap(),
            name: String::from_utf8_lossy(&tree.get("name")?.unwrap()[..]).to_string(),
            cloud: from_slice(&tree.get("cloud")?.unwrap()[..]).unwrap(),
            backlog: from_slice(&tree.get("backlog")?.unwrap()[..]).unwrap(),
            active: from_slice(&tree.get("active")?.unwrap()[..]).unwrap(),
            history: from_slice(&tree.get("history")?.unwrap()[..]).unwrap(),
        })
    }

    pub fn get_next_task(
        mut self,
    ) -> Result<Option<impl Future<Output = Result<Self, Error>> + Send>, Error> {
        match self.active {
            Some(a) => Err(Error::ExistingActiveTask(a)),
            None => match self.backlog.pop_front() {
                Some(item) => Ok(Some(async move {
                    let active = Active::from_queued(
                        item,
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos(),
                    );
                    let completed = self.cloud.handle_request(active).await?;
                    self.history.push(completed);
                    Ok(self)
                })),
                None => Ok(None),
            },
        }
    }

    pub fn get_state(&self) -> ModelState {
        let mut state: ModelState = Default::default();

        for item in &self.history {
            match &item.action {
                Action::CreateModel { name: _ } => state.status = ModelStatus::Ready,
                Action::ConfigureModel { foo } => state.config.foo = foo.clone(),
                _ => {}
            }
        }

        state
    }
}
