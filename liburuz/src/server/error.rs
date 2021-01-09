use crate::server::model::Active;
use k8s_openapi::RequestError as K8sError;
use kube::error::{Error as KubeError, ErrorResponse as KubeErrorResponse};
use serde_json::Error as SerdeJsonError;
use sled::transaction::TransactionError;
use sled::Error as SledError;
use std::io::Error as IOError;
use uuid::Error as UuidError;

#[derive(Debug)]
pub enum Error {
    IOError(IOError),
    UnknownCloud(String),
    UnexpectedShutdown(String),
    SledError(SledError),
    ModelLoad(String),
    ModelAlreadyExists(String),
    ModelAlreadyDeleted(String),
    SerdeJsonError(SerdeJsonError),
    K8sError(K8sError),
    KubeError(KubeError),
    KubeErrorResponse(KubeErrorResponse),
    ExistingActiveTask(Active),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IOError(err)
    }
}

impl From<SledError> for Error {
    fn from(err: SledError) -> Self {
        Error::SledError(err)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Self {
        Error::SerdeJsonError(err)
    }
}

impl From<K8sError> for Error {
    fn from(err: K8sError) -> Self {
        Error::K8sError(err)
    }
}

impl From<KubeError> for Error {
    fn from(err: KubeError) -> Self {
        Error::KubeError(err)
    }
}

impl From<KubeErrorResponse> for Error {
    fn from(err: KubeErrorResponse) -> Self {
        Error::KubeErrorResponse(err)
    }
}

impl From<UuidError> for Error {
    fn from(err: UuidError) -> Self {
        Error::ModelLoad(format!("Error loading UUID: {}", err))
    }
}

impl From<TransactionError<Error>> for Error {
    fn from(err: TransactionError<Error>) -> Self {
        match err {
            TransactionError::Abort(err) => err,
            TransactionError::Storage(err) => Error::SledError(err),
        }
    }
}
