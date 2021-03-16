pub mod aws;
pub mod dummy;
pub mod kubernetes;

use crate::server::error::Error;
use crate::server::model::{Action, Active, Completed};
use serde_derive::{Deserialize, Serialize};
use std::future::Future;
use std::string::ToString;
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
                (Self::AWS, Action::AddRune { name, rune }) => {
                    self::aws::add_rune(name, rune).await?
                }
                (
                    Self::AWS,
                    Action::ConfigureRune {
                        name,
                        attribute,
                        value,
                    },
                ) => self::aws::configure_rune(name, attribute, value).await?,
                (Self::AWS, Action::RemoveRune { name }) => self::aws::remove_rune(name).await?,
                (Self::Dummy, Action::CreateModel { name }) => {
                    self::dummy::create_model(name).await?
                }
                (Self::Dummy, Action::ConfigureModel { foo: _ }) => {
                    self::dummy::configure_model().await?
                }
                (Self::Dummy, Action::DestroyModel) => self::dummy::destroy_model().await?,
                (Self::Dummy, Action::AddRune { name, rune }) => {
                    self::dummy::add_rune(name, rune).await?
                }
                (
                    Self::Dummy,
                    Action::ConfigureRune {
                        name,
                        attribute,
                        value,
                    },
                ) => self::dummy::configure_rune(name, attribute, value).await?,
                (Self::Dummy, Action::RemoveRune { name }) => {
                    self::dummy::remove_rune(name).await?
                }
                (Self::Kubernetes, Action::CreateModel { name }) => {
                    self::kubernetes::create_model(name).await?
                }
                (Self::Kubernetes, Action::ConfigureModel { foo: _ }) => {
                    self::kubernetes::configure_model().await?
                }
                (Self::Kubernetes, Action::DestroyModel) => {
                    self::kubernetes::destroy_model().await?
                }
                (Self::Kubernetes, Action::AddRune { name, rune }) => {
                    self::kubernetes::add_rune(name, rune).await?
                }
                (
                    Self::Kubernetes,
                    Action::ConfigureRune {
                        name,
                        attribute,
                        value,
                    },
                ) => self::kubernetes::configure_rune(name, attribute, value).await?,
                (Self::Kubernetes, Action::RemoveRune { name }) => {
                    self::kubernetes::remove_rune(name).await?
                }
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

impl ToString for Cloud {
    fn to_string(&self) -> String {
        match self {
            Cloud::AWS => "aws".into(),
            Cloud::Dummy => "dummy".into(),
            Cloud::Kubernetes => "kubernetes".into(),
        }
    }
}
