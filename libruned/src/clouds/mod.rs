pub mod aws;
pub mod kubernetes;

use crate::error::Error;
use rune::Rune;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Cloud {
    AWS,
    Kubernetes,
}

impl Cloud {
    pub fn create_model(&self, name: &str) -> Result<(), Error> {
        Ok(())
    }

    pub fn from_name(name: &str) -> Result<Self, Error> {
        match name {
            "aws" => Ok(Self::AWS),
            "kubernetes" => Ok(Self::Kubernetes),
            _ => Err(Error::UnknownCloud(format!("Unknown cloud {}", name))),
        }
    }

    fn translate(&self, rune: Rune) -> () {
        match self {
            Self::AWS => {}
            Self::Kubernetes => {}
        }
    }
}
