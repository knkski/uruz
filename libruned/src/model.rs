use std::collections::HashMap;
use std::future::Future;
use std::str::from_utf8;

use crate::clouds::Cloud;
use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_slice;
use serde_json::to_vec;
use std::task::{Context, Poll};
use tokio::process::Command;
use uuid::Uuid;
use futures::future::TryFutureExt;

#[derive(Debug, Deserialize, Serialize)]
pub enum StateChange {
    CreateModel,
    ModelCreated,
    DestroyModel,
    ModelDestroyed,
    ConfigureModel,
    ModelConfigured,
    AddRune,
    RuneAdded,
    RemoveRune,
    RuneRemoved,
}

#[derive(Debug)]
pub struct ModelConfig {
    foo: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self { foo: "bar".into() }
    }
}

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct ModelState {
    status: ModelStatus,
    config: ModelConfig,
    runes: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct ModelTasks {
    state: Option<tokio::process::Child>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub cloud: Cloud,
    pub history: Vec<StateChange>,
    #[serde(skip)]
    tasks: ModelTasks,
}

impl Model {
    pub fn create(db: sled::Db, name: &str, cloud: Cloud) -> Result<Self, Error> {
        let model = Self {
            id: Uuid::new_v4(),
            name: name.into(),
            cloud,
            history: vec![StateChange::CreateModel],
            tasks: ModelTasks {
                ..Default::default()
            },
        };
        db.insert(model.id.as_bytes(), to_vec(&model)?)?;
        Ok(model)
    }

    pub fn load(db: sled::Db, name: sled::IVec, cloud: Option<Cloud>) -> Result<Self, Error> {
        let model: Model = match db.get(&name) {
            Ok(Some(bytes)) => from_slice(&bytes)?,
            Ok(None) => {
                Model::create(db.clone(), from_utf8(&name[..]).unwrap(), cloud.unwrap()).unwrap()
            }
            err => {
                return Err(Error::ModelLoad(format!(
                    "Error loading model `{:?}`: {:?} ",
                    name, err
                )))
            }
        };

        Ok(model)
    }

    fn desired_state(&self) -> ModelState {
        let mut state = ModelState {
            ..Default::default()
        };

        for item in &self.history {
            match item {
                StateChange::CreateModel => {
                    state.status = ModelStatus::Requested;
                }
                StateChange::ModelCreated => {
                    state.status = ModelStatus::Ready;
                }
                StateChange::DestroyModel => {}
                StateChange::ModelDestroyed => {}
                StateChange::ConfigureModel => {}
                StateChange::ModelConfigured => {}
                StateChange::AddRune => {}
                StateChange::RuneAdded => {}
                StateChange::RemoveRune => {}
                StateChange::RuneRemoved => {}
            }
        }

        state
    }

    async fn actual_state(&self) -> ModelState {
        let mut state = ModelState {
            ..Default::default()
        };
        let status = Command::new("microk8s.kubectl")
            .args(&["get", "namespace", &self.name])
            .output().await;
        state
    }

    async fn rectify_state(&self) {
        let expected_state = self.desired_state();
        let actual_state = self.actual_state().await;
    }
}

impl Future for Model {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        if let Some(state_task) = self.tasks.state.as_mut() {
            println!("Polling state task.");
            if let Poll::Ready(foo) = TryFutureExt::try_poll_unpin(state_task, ctx) {
                match foo {
                    Ok(exit_status) => {
                        println!("Got successful exit status {:?}", exit_status);
                    }
                    Err(exit_status) => {
                        println!("Got failing exit status {:?}", exit_status);
                    }
                }
                println!("Unloading state task.");
                self.tasks.state = None;
            }
        }
        ctx.waker().clone().wake();
        Poll::Pending
    }
}
