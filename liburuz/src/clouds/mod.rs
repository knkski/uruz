pub mod aws;
pub mod dummy;
pub mod kubernetes;

use crate::server::error::Error;
use crate::server::model::{Action, Active, Completed};
use serde_derive::{Deserialize, Serialize};
use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Cloud {
    AWS,
    Dummy,
    Kubernetes,
}

#[derive(Debug)]
pub enum ModelState {
    Requested,
    Creating,
    Ready,
    Configuring,
}

impl Cloud {
    pub fn from_str(name: &str) -> Result<Self, Error> {
        Cloud::from_bytes(name.as_bytes())
    }

    pub fn from_bytes(name: &[u8]) -> Result<Self, Error> {
        match name {
            b"aws" => Ok(Self::AWS),
            b"dummy" => Ok(Self::Dummy),
            b"kubernetes" => Ok(Self::Kubernetes),
            _ => Err(Error::UnknownCloud(format!("Unknown cloud {:?}", name))),
        }
    }

    pub fn handle_request(
        &self,
        request: Active,
    ) -> impl Future<Output = Result<Completed, Error>> {
        let cloud = self.clone();
        async move {
            match (cloud, request.get_action()) {
                (Self::AWS, Action::CreateModel { name }) => self::aws::create_model(name).await?,
                (Self::AWS, Action::ConfigureModel { foo: _ }) => {
                    self::aws::configure_model().await?
                }
                (Self::AWS, Action::DestroyModel) => self::aws::destroy_model().await?,
                (Self::AWS, Action::AddRune) => self::aws::add_rune().await?,
                (Self::AWS, Action::RemoveRune) => self::aws::remove_rune().await?,
                (Self::Dummy, Action::CreateModel { name }) => {
                    self::dummy::create_model(name).await?
                }
                (Self::Dummy, Action::ConfigureModel { foo: _ }) => {
                    self::dummy::configure_model().await?
                }
                (Self::Dummy, Action::DestroyModel) => self::dummy::destroy_model().await?,
                (Self::Dummy, Action::AddRune) => self::dummy::add_rune().await?,
                (Self::Dummy, Action::RemoveRune) => self::dummy::remove_rune().await?,
                (Self::Kubernetes, Action::CreateModel { name }) => {
                    self::kubernetes::create_model(name).await?
                }
                (Self::Kubernetes, Action::ConfigureModel { foo: _ }) => {
                    self::kubernetes::configure_model().await?
                }
                (Self::Kubernetes, Action::DestroyModel) => {
                    self::kubernetes::destroy_model().await?
                }
                (Self::Kubernetes, Action::AddRune) => self::kubernetes::add_rune().await?,
                (Self::Kubernetes, Action::RemoveRune) => self::kubernetes::remove_rune().await?,
            }

            Ok(Completed::from_active(
                request,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            ))
        }
    }
}
