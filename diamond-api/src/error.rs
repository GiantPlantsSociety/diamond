use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::time::{Duration, SystemTimeError};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    RenderFormat,
    SystemTimeError(Duration),
    ParseIntError(ParseIntError),
    EmptyString,
    Time,
    Query(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::RenderFormat => write!(f, "Format cannot be parsed"),
            ParseError::Time => write!(f, "Time cannot be parsed"),
            ParseError::SystemTimeError(d) => write!(
                f,
                "Second time provided was later than self in duration {:?}",
                d
            ),
            ParseError::ParseIntError(s) => write!(f, "{}", s),
            ParseError::EmptyString => write!(f, "Cannot parse empty string"),
            ParseError::Query(s) => write!(f, "{}", s),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseIntError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<SystemTimeError> for ParseError {
    fn from(error: SystemTimeError) -> Self {
        ParseError::SystemTimeError(error.duration())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        ParseError::ParseIntError(error)
    }
}

impl From<serde_urlencoded::de::Error> for ParseError {
    fn from(error: serde_urlencoded::de::Error) -> Self {
        ParseError::Query(error.to_string())
    }
}

impl actix_web::error::ResponseError for ParseError {}

#[derive(Debug, PartialEq)]
pub enum ResponseError {
    SystemTime(Duration),
    NotFound,
    Path,
    Kind(String),
}

impl Error for ResponseError {}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseError::NotFound => write!(f, "NotFound"),
            ResponseError::Path => write!(f, "PathError"),
            ResponseError::SystemTime(d) => write!(
                f,
                "Second time provided was later than self in duration {:?}",
                d
            ),
            ResponseError::Kind(s) => write!(f, "{}", s),
        }
    }
}

impl From<io::Error> for ResponseError {
    fn from(error: io::Error) -> Self {
        ResponseError::Kind(error.to_string())
    }
}

impl From<SystemTimeError> for ResponseError {
    fn from(error: SystemTimeError) -> Self {
        ResponseError::SystemTime(error.duration())
    }
}

impl actix_web::error::ResponseError for ResponseError {}
