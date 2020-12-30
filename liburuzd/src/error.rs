use crate::model::Active;
use k8s_openapi::RequestError as K8sError;
use kube::error::{Error as KubeError, ErrorResponse as KubeErrorResponse};
use serde_json::Error as SerdeJsonError;
use sled::Error as SledError;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    IOError(IOError),
    UnknownCloud(String),
    UnexpectedShutdown(String),
    SledError(SledError),
    ModelLoad(String),
    ModelAlreadyExists(String),
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
