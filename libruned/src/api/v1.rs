use crate::clouds::Cloud;
use crate::model::Model;
use serde_derive::{Deserialize, Serialize};
use serde_json::to_vec;
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct ModelCreate {
    name: String,
    cloud: String,
}

async fn list_models(db: sled::Db) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&vec![0u8]))
}

async fn get_model(name: String, db: sled::Db) -> Result<String, warp::Rejection> {
    Ok("foobar".into())
}

async fn create_model(
    db: sled::Db,
    args: ModelCreate,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Creating model {} on cloud {}", args.name, args.cloud);
    let cloud = match Cloud::from_name(&args.cloud) {
        Ok(c) => c,
        Err(_) => return Ok(warp::http::StatusCode::BAD_REQUEST),
    };
    tokio::spawn(Model::load(db, args.name[..].into(), Some(cloud)).unwrap());
    Ok(warp::http::StatusCode::OK)
}

pub fn build(
    db: sled::Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get_models_db = db.clone();
    let get_model_db = db.clone();
    let create_model_db = db.clone();
    warp::path!("api" / "v1" / "models")
        .and(warp::get())
        .and(warp::any().map(move || get_models_db.clone()))
        .and_then(list_models)
        .or(warp::path!("api" / "v1" / "models" / String)
            .and(warp::any().map(move || get_model_db.clone()))
            .and_then(get_model))
        .or(warp::path!("api" / "v1" / "models")
            .and(warp::post())
            .and(warp::any().map(move || create_model_db.clone()))
            .and(warp::body::json())
            .and_then(create_model))
}
