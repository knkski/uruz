use crate::error::Error;
use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, Client};

pub async fn create_model(name: &str) -> Result<(), Error> {
    let kube_client = Client::try_default().await.unwrap();
    let namespaces: Api<Namespace> = Api::all(kube_client);

    match namespaces.get(name).await {
        Ok(ns) => {
            println!(
                "Found namespace {:?}\n\n{:?}\n\n{:?}",
                ns, ns.metadata, ns.status
            );
            Err(Error::ModelAlreadyExists(
                "Namespace already exists!".into(),
            ))
        }
        Err(kube::Error::Api(err)) => match &err.reason[..] {
            "NotFound" => Ok(()),
            _ => {
                println!("Got unhandled error: {:?}", err);
                Err(err.into())
            }
        },
        Err(err) => {
            println!("Got unhandled error: {:?}", err);
            Err(err.into())
        }
    }
}
