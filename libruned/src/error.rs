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
    SerdeJsonError(SerdeJsonError),
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
