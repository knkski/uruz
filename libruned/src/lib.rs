pub mod api;
pub mod clouds;
pub mod config;
pub mod db;
pub mod error;
pub mod model;
pub mod traits;

use crate::config::Config;
use crate::error::Error;
use crate::model::Model;
use futures::{future::join_all, join};

pub async fn start(c: Config) -> Result<(), Error> {
    let db = sled::open(c.database_path)?;
    let api_v1 = api::v1::build(db.clone());

    let api = warp::serve(api_v1).run((c.api_host, c.api_port));

    let processors: Result<Vec<_>, _> = db
        .iter()
        .map(|item| {
            let name = match item {
                Ok(n) => n.0,
                err => return Err(Error::ModelLoad(format!("Couldn't load model: {:?}", err))),
            };
            Ok(Model::load(db.clone(), name, None)?)
        })
        .collect();

    let loaded_processors = processors?;

    join!(api, join_all(loaded_processors));

    Err(Error::UnexpectedShutdown(
        "No futures left to process. This should not happen!".into(),
    ))
}
