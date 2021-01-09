use crate::api::v1::{Model, ModelConfigure, ModelCreate};
use crate::client::error::Error;
use async_std::task;
use std::time::Duration;
use uuid::Uuid;

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
            .get(&format!("{}/api/v1/models/{}", self.endpoint, model_id))
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

    pub async fn destroy_model(&self, model_id: &str) -> Result<Uuid, Error> {
        Ok(self
            .req
            .delete(&format!("{}/api/v1/models/{}", self.endpoint, model_id))
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
        let action_id = self.configure_model(model_id, args).await?;
        self.wait_for_action(model_id, action_id).await?;
        self.get_model(model_id).await
    }

    pub async fn destroy_model_wait(&self, model_id: &str) -> Result<(), Error> {
        let action_id = self.destroy_model(model_id).await?;
        self.wait_for_action(model_id, action_id).await?;
        Ok(())
    }

    pub async fn wait_for_action(&self, model_id: &str, uuid: Uuid) -> Result<(), Error> {
        for _ in 0u32..10 {
            let model = self.get_model(model_id).await?;

            if model
                .actions
                .iter()
                .any(|a| a.id == uuid && a.completed.is_some())
            {
                return Ok(());
            } else {
                task::sleep(Duration::from_secs(1)).await;
            }
        }
        Err(Error::TimeoutError(uuid))
    }
}
