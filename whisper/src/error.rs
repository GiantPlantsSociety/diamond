use std::fmt::{Display, Formatter, Result};
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileNotExist(PathBuf),
    Kind(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::FileNotExist(e) => write!(f, "[ERROR] File {:#?} does not exist!", e),
            Error::Kind(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
#[derive(Debug, PartialEq)]
pub enum ParseError {
    ParsePointError(String),
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ParseError::ParsePointError(e) => write!(f, "Cannot parse point from string: {}", e),
            ParseError::ParseFloatError(e) => write!(f, "cause: {}", e),
            ParseError::ParseIntError(e) => write!(f, "cause: {}", e),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseFloatError(e) => Some(e),
            Self::ParseIntError(e) => Some(e),
            _ => None,
        }
    }
}
