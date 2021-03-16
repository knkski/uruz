use crate::clouds::Cloud;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCreate {
    pub name: String,
    pub cloud: Cloud,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigure {
    pub foo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuneAdd {
    pub name: String,
    pub rune: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuneConfigure {
    pub attribute: String,
    pub value: String,
}
