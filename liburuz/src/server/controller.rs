use crate::clouds::Cloud;
use crate::rune::v1::rune::Rune;
use crate::server::error::Error;
use crate::server::model::{Action, Active, Completed, Model, ModelStatus, Queued};
use async_std::task;
use rustpython_vm::{
    builtins::{PyBaseObject, PyStr},
    compile::Mode as CompileMode,
    exceptions::{write_exception, PyBaseException},
    py_freeze,
    pyobject::{PyRef, PyResult},
    scope::Scope,
    InitParameter, Interpreter, PySettings,
};
use serde_json::{from_slice, to_vec};
use sled::transaction::abort;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type ModelState = (Vec<Completed>, Option<Active>, VecDeque<Queued>);
type R = Result<Option<Completed>, Error>;

fn map_vm_err<'a>(vm: &'a Interpreter) -> impl Fn(PyRef<PyBaseException>) -> Error + 'a {
    move |err| {
        let tb = vm.enter(|vm| -> Result<String, fmt::Error> {
            let mut rendered = String::new();
            write_exception(&mut rendered, vm, &err)?;
            Ok(rendered)
        });
        match tb {
            Ok(tb) => Error::VMInitError(tb),
            Err(err) => Error::VMInitError(format!("Error handling VM init error: {}", err)),
        }
    }
}

#[derive(Clone)]
pub struct Controller {
    database: sled::Db,
    futures: Arc<Mutex<HashMap<Uuid, Pin<Box<dyn Future<Output = R> + Send>>>>>,
    vm: Arc<Mutex<Interpreter>>,
    scopes: Arc<Mutex<HashMap<(Uuid, String), Scope>>>,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut settings = PySettings::default();
        settings.path_list = vec!["/home/knkski/Sources/RustPython/Lib".into()];
        let python = Interpreter::new_with_init(settings, |vm| {
            vm.add_frozen(py_freeze!(dir = "../python/"));
            // TODO: Use this instead of path_list above. This is slow to compile, though
            // vm.add_frozen(py_freeze!(
            //     dir = "/home/knkski/Sources/RustPython/Lib",
            // ));
            InitParameter::External
        });

        Ok(Self {
            database: sled::open(path)?,
            futures: Arc::new(Mutex::new(HashMap::new())),
            vm: Arc::new(Mutex::new(python)),
            scopes: Arc::new(Mutex::new(HashMap::new())),
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
        model_vm: &Interpreter,
        scopes: &HashMap<(Uuid, String), Scope>,
        runes: &HashMap<&str, &Rune>,
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
                if let Action::ConfigureRune {
                    name,
                    attribute,
                    value,
                } = &q.action
                {
                    let scope = scopes.get(&(model_id.clone(), name.clone())).unwrap();
                    let rune = runes.get(name.as_str()).unwrap();
                    let next = self.eval_action(model_vm, scope.clone(), rune, &q)?;
                }
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

    fn init_vm_rune(
        &self,
        vm: &Interpreter,
        name: String,
        rune: &Rune,
        scope: Scope,
    ) -> Result<(), Error> {
        vm.enter(|vm| -> PyResult<()> {
            let code_obj = vm
                .compile(rune.react.as_ref().unwrap(), CompileMode::Exec, name)
                .map_err(|err| vm.new_syntax_error(&err))?;

            vm.run_code_obj(code_obj, scope.clone())?;
            Ok(())
        })
        .map_err(map_vm_err(vm))?;

        Ok(())
    }

    fn eval_action(
        &self,
        vm: &Interpreter,
        scope: Scope,
        rune: &Rune,
        action: &Queued,
    ) -> Result<(), Error> {
        vm.enter(|vm| -> PyResult<String> {
            let code_obj = vm
                .compile(
                    &format!("{}({{}})", rune.metadata.react.as_ref().unwrap()),
                    CompileMode::Eval,
                    "<rune>".to_owned(),
                )
                .map_err(|err| vm.new_syntax_error(&err))?;

            vm.run_code_obj(code_obj, scope).and_then(|obj| {
                let name = vm.get_attribute(obj, "name")?;
                Ok(name.payload::<PyStr>().unwrap().as_ref().to_owned())
            })
        })
        .map_err(map_vm_err(vm))?;
        Ok(())
    }
}

impl Future for Controller {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let futures = self.futures.clone();
        let mut tasks = futures.lock().unwrap();

        let vm = self.vm.clone();
        let vm = vm.lock().unwrap();

        let scopes = self.scopes.clone();
        let mut scopes = scopes.lock().unwrap();

        // Synchronize in-memory tasks with what's on disk
        for id in self.database.tree_names() {
            if id == b"__sled__default" {
                continue;
            }
            let id = Uuid::from_slice(&id).unwrap();
            let model = self.get_model(&id).unwrap();
            let runes = model.get_runes();

            for (&name, &rune) in &runes {
                let key = (id, name.to_owned());
                if !scopes.contains_key(&key) {
                    let scope = vm.enter(|vm| vm.new_scope_with_builtins());
                    scopes.insert(key, scope.clone());
                    self.init_vm_rune(&vm, name.to_owned(), &rune, scope)
                        .unwrap();
                }
            }

            if tasks.contains_key(&id) {
                if model.get_status() == ModelStatus::Destroyed {
                    tasks.remove(&id);
                }
            } else {
                if let Ok(foo) = self.get_next_task(&id, None, &vm, &scopes, &runes) {
                    tasks.insert(id, Box::pin(*foo));
                }
            }
        }

        for (mid, task) in tasks.iter_mut() {
            if let Poll::Ready(result) = task.as_mut().poll(ctx) {
                match result {
                    Ok(completed) => {
                        let model = self.get_model(&mid).unwrap();
                        let runes = model.get_runes();
                        let next = self
                            .get_next_task(mid, completed, &vm, &scopes, &runes)
                            .unwrap();
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
