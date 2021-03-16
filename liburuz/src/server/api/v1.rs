use crate::api::v1;
use crate::rune::v1::rune::Rune;
use crate::server::controller::Controller;
use crate::server::model::Action;
use uuid::Uuid;
use warp::Filter;

async fn list_models(_controller: Controller) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&vec![0u8]))
}

async fn get_model(
    id: String,
    controller: Controller,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.get_model(&Uuid::parse_str(&id).unwrap()) {
        Ok(model) => Ok(warp::reply::json::<v1::Model>(&model.into())),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn create_model(
    mut controller: Controller,
    args: v1::ModelCreate,
) -> Result<impl warp::Reply, warp::Rejection> {
    match controller.create_model(args.cloud, &args.name) {
        Ok(model) => Ok(warp::reply::json::<v1::Model>(&model.into())),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn configure_model(
    id: String,
    controller: Controller,
    conf: v1::ModelConfigure,
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

async fn add_rune(
    id: String,
    controller: Controller,
    args: v1::RuneAdd,
) -> Result<impl warp::Reply, warp::Rejection> {
    let rune = Rune::unzip(&args.rune).unwrap();
    match controller.add_rune(&Uuid::parse_str(&id).unwrap(), args.name, rune) {
        Ok(id) => Ok(warp::reply::json(&id)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn configure_rune(
    model_id: String,
    rune_name: String,
    controller: Controller,
    args: v1::RuneConfigure,
) -> Result<impl warp::Reply, warp::Rejection> {
    let model_id = Uuid::parse_str(&model_id).unwrap();
    let result = controller.update_model(
        &model_id,
        Action::ConfigureRune {
            name: rune_name,
            attribute: args.attribute,
            value: args.value,
        },
    );
    match result {
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
        .or(warp::path!("api" / "v1" / "models" / String / "runes")
            .and(warp::post())
            .and(controller.clone())
            .and(warp::body::json())
            .and_then(add_rune))
        .or(
            warp::path!("api" / "v1" / "models" / String / "runes" / String / "config")
                .and(warp::patch())
                .and(controller.clone())
                .and(warp::body::json())
                .and_then(configure_rune),
        )
}
