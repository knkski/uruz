use serde_derive::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum TemplateInteger {
    Integer(u32),
    Template(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum Image {
    Source { source: String },
    Build { build: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Port {
    name: String,
    #[serde(rename = "containerPort")]
    container_port: TemplateInteger,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Template {
    pub name: String,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    pub image: Image,
    pub ports: Vec<Port>,
    pub include: Option<Value>,
}
