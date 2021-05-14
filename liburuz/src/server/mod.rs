pub mod api;
pub mod config;
pub mod controller;
pub mod error;
pub mod model;

use self::config::Config;
use self::controller::Controller;
use self::error::Error;
use std::future::Future;

use futures::join;

pub fn start(c: Config) -> Result<impl Future<Output = Result<(), Error>>, Error> {
    let controller = Controller::new(c.database_path)?;
    let api_v1 = api::v1::build(controller.clone());

    let api = warp::serve(api_v1).run((c.api_host, c.api_port));

    Ok(async {
        join!(api, controller);

        Err(Error::UnexpectedShutdown(
            "No futures left to process. This should not happen!".into(),
        ))
    })
}
