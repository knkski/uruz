//! V1 API

use crate::clouds::Cloud;
use crate::server::model;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ActionKind {
    CreateModel { name: String },
    ConfigureModel { foo: String },
    DestroyModel,
    AddRune,
    RemoveRune,
}

impl ActionKind {
    fn from_action(a: &model::Action) -> Self {
        match a {
            model::Action::CreateModel { name } => ActionKind::CreateModel { name: name.clone() },
            model::Action::ConfigureModel { foo } => {
                ActionKind::ConfigureModel { foo: foo.clone() }
            }
            model::Action::DestroyModel => ActionKind::DestroyModel,
            model::Action::AddRune => ActionKind::AddRune,
            model::Action::RemoveRune => ActionKind::RemoveRune,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Action {
    pub id: Uuid,
    pub kind: ActionKind,
    pub queued: u128,
    pub started: Option<u128>,
    pub completed: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub foo: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelState {
    pub config: ModelConfig,
    pub runes: HashMap<String, String>,
}

impl ModelState {
    fn from_state(s: &model::ModelState) -> Self {
        Self {
            config: ModelConfig {
                foo: s.config.foo.clone(),
            },
            runes: s.runes.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub cloud: String,
    pub actions: Vec<Action>,
    pub state: ModelState,
}

impl Model {
    pub fn from_model(model: &model::Model) -> Self {
        let mut actions: Vec<_> = model
            .history
            .iter()
            .map(|h| Action {
                id: h.id,
                kind: ActionKind::from_action(&h.action),
                queued: h.queued,
                started: Some(h.started),
                completed: Some(h.completed),
            })
            .collect();
        if let Some(a) = &model.active {
            actions.push(Action {
                id: a.id,
                kind: ActionKind::from_action(&a.action),
                queued: a.queued,
                started: Some(a.started),
                completed: None,
            });
        }
        actions.extend(model.backlog.iter().map(|h| Action {
            id: h.id,
            kind: ActionKind::from_action(&h.action),
            queued: h.queued,
            started: None,
            completed: None,
        }));
        Self {
            id: model.id.to_string(),
            name: model.name.clone(),
            cloud: format!("{:?}", model.cloud),
            actions,
            state: ModelState::from_state(&model.get_state()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCreate {
    pub name: String,
    pub cloud: Cloud,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigure {
    pub foo: String,
}
