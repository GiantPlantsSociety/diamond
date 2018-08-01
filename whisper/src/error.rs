use failure::*;
use std::path::PathBuf;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
    #[fail(display = "[ERROR] File {:#?} does not exist!", _0)]
    FileNotExist(PathBuf),
}

#[derive(Debug, PartialEq, Fail)]
pub enum ParseError {
    #[fail(display = "Cannot parse point from string: {}", _0)]
    ParsePointError(String),
    #[fail(display = "{}", _0)]
    ParseFloatError(#[cause] ::std::num::ParseFloatError),
    #[fail(display = "{}", _0)]
    ParseIntError(#[cause] ::std::num::ParseIntError),
}
