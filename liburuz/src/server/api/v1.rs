use crate::api::v1::{Model as ApiModel, ModelConfigure, ModelCreate};
use crate::server::controller::Controller;
use crate::server::model::Action;
use uuid::Uuid;
use warp::Filter;

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ApiModel {
//     pub id: String,
//     pub name: String,
//     pub cloud: String,
//     pub backlog: Vec<Queued>,
//     pub active: Option<Active>,
//     pub history: Vec<Completed>,
//     pub state: ModelState,
// }

// impl ApiModel {
//     fn from_model(model: &Model) -> Self {
//         Self {
//             id: model.id.to_string(),
//             name: model.name.clone(),
//             cloud: format!("{:?}", model.cloud),
//             backlog: model.backlog.iter().cloned().collect(),
//             active: model.active.clone(),
//             history: model.history.clone(),
//             state: model.get_state(),
//         }
//     }
// }

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ModelCreate {
//     pub name: String,
//     pub cloud: String,
// }

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ModelConfigure {
//     pub foo: String,
// }

async fn list_models(_controller: Controller) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&vec![0u8]))
}

async fn get_model(
    id: String,
    controller: Controller,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.get_model(&Uuid::parse_str(&id).unwrap()) {
        Ok(model) => Ok(warp::reply::json(&ApiModel::from_model(&model))),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn create_model(
    mut controller: Controller,
    args: ModelCreate,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.create_model(args.cloud, &args.name) {
        Ok(model) => Ok(warp::reply::json(&ApiModel::from_model(&model))),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn configure_model(
    id: String,
    controller: Controller,
    conf: ModelConfigure,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.update_model(
        &Uuid::parse_str(&id).unwrap(),
        Action::ConfigureModel { foo: conf.foo },
    ) {
        Ok(id) => Ok(warp::reply::json(&id)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn delete_model(
    id: String,
    controller: Controller,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.delete_model(&Uuid::parse_str(&id).unwrap()) {
        Ok(id) => Ok(warp::reply::json(&id)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

pub fn build(
    controller: Controller,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let controller = warp::any().map(move || controller.clone());

    warp::path!("api" / "v1" / "models")
        .and(warp::get())
        .and(controller.clone())
        .and_then(list_models)
        .or(warp::path!("api" / "v1" / "models" / String)
            .and(warp::get())
            .and(controller.clone())
            .and_then(get_model))
        .or(warp::path!("api" / "v1" / "models")
            .and(warp::post())
            .and(controller.clone())
            .and(warp::body::json())
            .and_then(create_model))
        .or(warp::path!("api" / "v1" / "models" / String / "config")
            .and(warp::post())
            .and(controller.clone())
            .and(warp::body::json())
            .and_then(configure_model))
        .or(warp::path!("api" / "v1" / "models" / String)
            .and(warp::delete())
            .and(controller.clone())
            .and_then(delete_model))
}
