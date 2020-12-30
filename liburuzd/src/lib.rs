pub mod api;
pub mod clouds;
pub mod config;
pub mod controller;
pub mod db;
pub mod error;
pub mod model;
pub mod traits;

use crate::config::Config;
use crate::controller::Controller;
use crate::error::Error;

use futures::join;

pub async fn start(c: Config) -> Result<(), Error> {
    let controller = Controller::new(c.database_path)?;
    let api_v1 = api::v1::build(controller.clone());

    let api = warp::serve(api_v1).run((c.api_host, c.api_port));

    join!(api, controller);

    Err(Error::UnexpectedShutdown(
        "No futures left to process. This should not happen!".into(),
    ))
}
