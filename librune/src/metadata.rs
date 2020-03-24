use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::from_slice;
use std::collections::HashMap;
use std::fs::read;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum SecretSource {
    Generate,
    Env { file: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum ConfigItem {
    Boolean {
        description: String,
        default: bool,
        transformer: Option<String>,
    },
    Integer {
        description: String,
        default: u32,
        transformer: Option<String>,
    },
    String {
        description: String,
        default: String,
        transformer: Option<String>,
    },
    Secret {
        description: String,
        source: Option<SecretSource>,
        transformer: Option<String>,
    },
    Archive {
        description: String,
        transformer: Option<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provide {
    name: String,
    interface: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Require {
    name: String,
    interface: String,
    min: Option<u32>,
    max: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub repo: String,
    pub maintainers: Vec<String>,
    pub tags: Vec<String>,
    pub series: Vec<String>,
    #[serde(default)]
    pub provides: Vec<Provide>,
    #[serde(default)]
    pub requires: Vec<Require>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    pub react: Option<String>,
    pub config: HashMap<String, ConfigItem>,
}

impl Metadata {
    pub fn from_slice<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        Ok(from_slice(&read(path.into())?)?)
    }
}
