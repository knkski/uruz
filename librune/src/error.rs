use serde_yaml::Error as YamlError;
use std::io::Error as IOError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
    IOError(IOError),
    YamlError(YamlError),
    ZipError(ZipError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IOError(err)
    }
}

impl From<YamlError> for Error {
    fn from(err: YamlError) -> Self {
        Error::YamlError(err)
    }
}

impl From<ZipError> for Error {
    fn from(err: ZipError) -> Self {
        Error::ZipError(err)
    }
}
