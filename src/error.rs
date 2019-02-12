use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    Unknown,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind: kind }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "path error")
    }
}

impl StdError for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::new(ErrorKind::Io(error))
    }
}
