use crate::clouds::Cloud;
use crate::error::Error;
use crate::model::{Action, Model, Queued};
use async_std::task;
use serde_json::to_vec;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct Controller {
    database: sled::Db,
    futures: Arc<Mutex<HashMap<Uuid, Pin<Box<dyn Future<Output = Result<Model, Error>> + Send>>>>>,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            database: sled::open(path)?,
            futures: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn create_model(&mut self, cloud: &str, name: &str) -> Result<Model, Error> {
        let cloud = Cloud::from_name(cloud)?;

        for tree_name in self.database.tree_names() {
            if self.database.open_tree(tree_name)?.get("name")? == Some(name.into()) {
                return Err(Error::ModelAlreadyExists(name.into()));
            }
        }

        let model = Model::with_name(name.to_string(), cloud);
        self.save_model(&model)?;
        self.get_model(&model.id)
    }

    fn save_model(&self, model: &Model) -> Result<(), Error> {
        match self.database.open_tree(model.id.as_bytes()) {
            Ok(tree) => {
                tree.transaction(
                    |t| -> sled::transaction::ConflictableTransactionResult<(), Error> {
                        t.insert("id", model.id.as_bytes())?;
                        t.insert("name", to_vec(&model.name).unwrap())?;
                        t.insert("cloud", to_vec(&model.cloud).unwrap())?;
                        t.insert("backlog", to_vec(&model.backlog).unwrap())?;
                        t.insert("active", to_vec(&model.active).unwrap())?;
                        t.insert("history", to_vec(&model.history).unwrap())?;
                        Ok(())
                    },
                )
                .unwrap();
            }
            Err(_) => unreachable!(),
        };
        Ok(())
    }

    pub fn get_model(&self, id: &Uuid) -> Result<Model, Error> {
        match self.database.open_tree(id.as_bytes()) {
            Ok(tree) => Model::from_tree(tree),
            Err(_) => Err(Error::ModelLoad(id.to_simple().to_string())),
        }
    }

    pub fn update_model(&self, id: &Uuid, action: Action) -> Result<Uuid, Error> {
        let mut model = self.get_model(id)?;
        model.backlog.push_back(Queued::from_action(action));
        self.save_model(&model)?;
        Ok(model.backlog.back().unwrap().id)
    }

    pub async fn run(&self) {
        loop {
            task::sleep(Duration::from_secs(1)).await;
        }
    }
}

impl Future for Controller {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let futures = self.futures.clone();
        let mut tasks = futures.lock().unwrap();

        for id in self.database.tree_names() {
            if id == b"__sled__default" {
                continue;
            }
            let id = Uuid::from_slice(&id).unwrap();
            if !tasks.contains_key(&id) {
                if let Ok(Some(task)) = self.get_model(&id).unwrap().get_next_task() {
                    tasks.insert(id, Box::pin(task));
                }
            }
        }

        for task in tasks.values_mut() {
            if let Poll::Ready(result) = task.as_mut().poll(ctx) {
                match result {
                    Ok(new_model) => {
                        let foo = new_model.clone();
                        self.save_model(&new_model).unwrap();
                        if let Ok(Some(t)) = new_model.get_next_task() {
                            *task = Box::pin(t);
                        } else {
                            *task = Box::pin(async { Ok(foo) })
                        }
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
        }

        ctx.waker().wake_by_ref();
        Poll::Pending
    }
}
