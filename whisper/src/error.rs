#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
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

// type Result<T> = ::std::result::Result<T, Error>;
