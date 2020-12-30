use crate::error::Error;
use async_std::task;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// Model stuff

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Active {
    id: Uuid,
    action: Action,
    started: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Completed {
    id: Uuid,
    action: Action,
    started: u128,
    completed: u128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub foo: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Requested,
    Creating,
    Ready,
    Configuring,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelState {
    pub status: ModelStatus,
    pub config: ModelConfig,
    pub runes: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub cloud: String,
    pub backlog: Vec<Queued>,
    pub active: Option<Active>,
    pub history: Vec<Completed>,
    pub state: ModelState,
}

// Actions

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCreate {
    pub name: String,
    pub cloud: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigure {
    pub foo: String,
}

// Client

pub struct Client {
    endpoint: String,
    req: reqwest::Client,
}

impl Client {
    pub fn new<S: Into<String>>(endpoint: S) -> Self {
        Self {
            endpoint: endpoint.into(),
            req: reqwest::Client::new(),
        }
    }

    pub async fn create_model(&self, args: &ModelCreate) -> Result<Model, Error> {
        Ok(self
            .req
            .post(&format!("{}/api/v1/models", self.endpoint))
            .json(args)
            .send()
            .await?
            .json()
            .await?)
    }
    pub async fn get_model(&self, model_id: &str) -> Result<Model, Error> {
        Ok(self
            .req
            .post(&format!("{}/api/v1/models/{}", self.endpoint, model_id))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn configure_model(
        &self,
        model_id: &str,
        args: &ModelConfigure,
    ) -> Result<Uuid, Error> {
        Ok(self
            .req
            .post(&format!(
                "{}/api/v1/models/{}/config",
                self.endpoint, model_id
            ))
            .json(args)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn configure_model_wait(
        &self,
        model_id: &str,
        args: &ModelConfigure,
    ) -> Result<Model, Error> {
        let action_id = self.configure_model(model_id, args).await.unwrap();
        self.wait_for_action(model_id, action_id).await.unwrap();
        self.get_model(model_id).await
    }

    pub async fn wait_for_action(&self, model_id: &str, uuid: Uuid) -> Result<(), Error> {
        for _ in 0u32..10 {
            let model = self.get_model(model_id).await?;

            if model.history.iter().any(|h| h.id == uuid) {
                return Ok(());
            } else {
                task::sleep(Duration::from_secs(1)).await;
            }
        }
        Err(Error::TimeoutError(uuid))
    }
}
