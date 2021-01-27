use crate::api::v1::{Model, ModelConfigure, ModelCreate, RuneAdd};
use crate::client::error::Error;
use crate::rune::v1::rune::Rune;
use async_std::task;
use reqwest::{Method, RequestBuilder};
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

    async fn send<T, F>(&self, method: Method, path: &str, modifier: F) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        F: Fn(RequestBuilder) -> RequestBuilder,
    {
        let mut builder = self
            .req
            .request(method, &format!("{}/api/v1/models/{}", self.endpoint, path));
        builder = modifier(builder);
        Ok(builder.send().await?.error_for_status()?.json().await?)
    }

    pub async fn create_model(&self, args: &ModelCreate) -> Result<Model, Error> {
        self.send(Method::POST, "", |r| r.json(args)).await
    }

    pub async fn get_model(&self, model_id: &str) -> Result<Model, Error> {
        self.send(Method::GET, model_id, |r| r).await
    }

    pub async fn configure_model(
        &self,
        model_id: &str,
        args: &ModelConfigure,
    ) -> Result<Uuid, Error> {
        self.send(Method::POST, &format!("{}/config", model_id), |r| {
            r.json(args)
        })
        .await
    }

    pub async fn destroy_model(&self, model_id: &str) -> Result<Uuid, Error> {
        self.send(Method::DELETE, model_id, |r| r).await
    }

    pub async fn add_rune(&self, model_id: &str, name: &str, rune: &Rune) -> Result<Uuid, Error> {
        self.send(Method::POST, &format!("{}/rune", model_id), |r| {
            r.json(&RuneAdd {
                name: name.into(),
                rune: rune.zip().unwrap(),
            })
        })
        .await
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

    pub async fn add_rune_wait(
        &self,
        model_id: &str,
        name: &str,
        rune: &Rune,
    ) -> Result<(), Error> {
        let action_id = self.add_rune(model_id, name, rune).await?;
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
