use crate::clouds::Cloud;
use crate::rune::v1::rune::Rune;
use crate::server::error::Error;
use crate::server::model::{Action, Active, Completed, Model, ModelStatus, Queued};
use async_std::task;
use serde_json::{from_slice, to_vec};
use sled::transaction::abort;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type ModelState = (Vec<Completed>, Option<Active>, VecDeque<Queued>);

#[derive(Clone)]
pub struct Controller {
    database: sled::Db,
    futures: Arc<
        Mutex<
            HashMap<Uuid, Pin<Box<dyn Future<Output = Result<Option<Completed>, Error>> + Send>>>,
        >,
    >,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            database: sled::open(path)?,
            futures: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn create_model(&mut self, cloud: Cloud, name: &str) -> Result<Model, Error> {
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
        let tree = self.database.open_tree(model.id.as_bytes())?;
        tree.transaction(|t| {
            if t.get("id")?.is_some() {
                return abort(Error::ModelAlreadyExists(model.id.to_string()));
            }
            t.insert("id", model.id.as_bytes())?;
            t.insert("name", to_vec(&model.name).unwrap())?;
            t.insert("cloud", to_vec(&model.cloud).unwrap())?;
            t.insert("backlog", to_vec(&model.backlog).unwrap())?;
            t.insert("active", to_vec(&model.active).unwrap())?;
            t.insert("history", to_vec(&model.history).unwrap())?;
            Ok(())
        })
        .unwrap();
        Ok(())
    }

    fn get_cloud(&self, model_id: &Uuid) -> Result<Cloud, Error> {
        let tree = self.database.open_tree(model_id.as_bytes())?;
        Ok(from_slice(&tree.get("cloud")?.unwrap())?)
    }

    fn transaction<F>(&self, model_id: &Uuid, func: F) -> Result<ModelState, Error>
    where
        F: Fn(ModelState) -> Result<ModelState, Error>,
    {
        let tree = self.database.open_tree(model_id.as_bytes())?;
        Ok(tree.transaction(|t| {
            let history: Vec<Completed> = from_slice(&t.get("history")?.unwrap()).unwrap();
            let backlog = from_slice(&t.get("backlog")?.unwrap()).unwrap();
            let active = from_slice(&t.get("active")?.unwrap()).unwrap();
            let model_destroyed = history.iter().any(|h| h.action == Action::DestroyModel);
            if model_destroyed {
                return abort(Error::ModelAlreadyDeleted("".into()));
            }
            let (history, active, backlog) = func((history, active, backlog)).unwrap();
            t.insert("history", to_vec(&history).unwrap())?;
            t.insert("active", to_vec(&active).unwrap())?;
            t.insert("backlog", to_vec(&backlog).unwrap())?;
            Ok((history, active, backlog))
        })?)
    }

    fn add_to_backlog(&self, model_id: &Uuid, queued: Queued) -> Result<Uuid, Error> {
        self.transaction(model_id, |(history, active, mut backlog)| {
            backlog.push_back(queued.clone());
            Ok((history, active, backlog))
        })?;

        Ok(queued.id)
    }

    fn get_next_task(
        &self,
        model_id: &Uuid,
        completed: Option<Completed>,
    ) -> Result<Box<impl Future<Output = Result<Option<Completed>, Error>> + Send>, Error> {
        let (_, active, _) = self.transaction(model_id, |(mut history, active, mut backlog)| {
            match (&active, &completed) {
                (Some(a), Some(c)) => assert_eq!(a.id, c.id),
                _ => {}
            }
            if let Some(c) = &completed {
                history.push(c.clone());
            }
            if let Some(q) = backlog.pop_front() {
                let active = Active::from_queued(
                    q,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                );
                return Ok((history, Some(active), backlog));
            } else {
                return Ok((history, None, backlog));
            }
        })?;

        let cloud = self.get_cloud(model_id)?;
        Ok(Box::new(async move {
            if let Some(a) = active {
                Ok(Some(cloud.handle_request(a).await?))
            } else {
                task::sleep(Duration::from_secs(1)).await;
                Ok(None)
            }
        }))
    }

    pub fn get_model(&self, id: &Uuid) -> Result<Model, Error> {
        match self.database.open_tree(id.as_bytes()) {
            Ok(tree) => Model::from_tree(&tree),
            Err(_) => Err(Error::ModelLoad(id.to_simple().to_string())),
        }
    }

    pub fn update_model(&self, id: &Uuid, action: Action) -> Result<Uuid, Error> {
        let queued = Queued::from_action(
            action,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        self.add_to_backlog(id, queued)
    }

    pub fn delete_model(&self, id: &Uuid) -> Result<Uuid, Error> {
        let queued = Queued::from_action(
            Action::DestroyModel,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        self.add_to_backlog(id, queued)
    }

    pub fn add_rune(&self, id: &Uuid, name: String, rune: Rune) -> Result<Uuid, Error> {
        let queued = Queued::from_action(
            Action::AddRune { name, rune },
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        self.add_to_backlog(id, queued)
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
            let model = self.get_model(&id).unwrap();
            if tasks.contains_key(&id) {
                if model.get_status() == ModelStatus::Destroyed {
                    tasks.remove(&id);
                }
            } else if let Ok(foo) = self.get_next_task(&id, None) {
                tasks.insert(id, Box::pin(*foo));
            }
        }

        for (mid, task) in tasks.iter_mut() {
            if let Poll::Ready(result) = task.as_mut().poll(ctx) {
                match result {
                    Ok(completed) => {
                        let next = self.get_next_task(mid, completed).unwrap();
                        *task = Box::pin(*next);
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
