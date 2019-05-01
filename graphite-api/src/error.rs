use std::convert::From;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::time::{Duration, SystemTimeError};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    RenderFormat,
    SystemTimeError(Duration),
    ParseIntError(ParseIntError),
    EmptyString,
    Time,
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
        }
    }
}

impl Error for ParseError {}

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
