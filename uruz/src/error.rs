use liburuz::rune::Error as RuneError;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    IOError(IOError),
    RuneError(RuneError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IOError(err)
    }
}

impl From<RuneError> for Error {
    fn from(err: RuneError) -> Self {
        Error::RuneError(err)
    }
}
