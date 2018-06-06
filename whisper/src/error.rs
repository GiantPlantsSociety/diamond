#[derive(Debug, Fail)]
pub enum InvalidConfiguration {
    #[fail(display = "You must specify at least one archive configuration!")]
    NoArchives,

    #[fail(display = "A Whisper database may not be configured having two archives with the same precision (at index {}, {:?} and next is {:?})", _0, _1, _2)]
    SamePrecision(usize, ::archive_info::ArchiveInfo, ::archive_info::ArchiveInfo),

    #[fail(display = "Higher precision archives' precision must evenly divide all lower precision archives' precision (at index {}, {:?} and next is {:?})", _0, _1, _2)]
    UndividablePrecision(usize, ::archive_info::ArchiveInfo, ::archive_info::ArchiveInfo),

    #[fail(display = "Lower precision archives must cover larger time intervals than higher precision archives (at index {}: {} seconds and next is {} seconds)", _0, _1, _2)]
    BadRetention(usize, u32, u32),

    #[fail(display = "Each archive must have at least enough points to consolidate to the next archive (archive at index {} consolidates {} of previous archive's points but it has only {} total points)", _0, _1, _2)]
    NotEnoughPoints(usize, u32, u32),

    #[fail(display = "Invalid xFilesFactor {}, not between 0 and 1", _0)]
    InvalidXFilesFactor(f32),
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Cannot parse point from string: {}", _0)]
    ParsePointError(String),
    #[fail(display = "{}", _0)]
    ParseFloatError(#[cause] ::std::num::ParseFloatError),
    #[fail(display = "{}", _0)]
    ParseIntError(#[cause] ::std::num::ParseIntError),
}

// type Result<T> = ::std::result::Result<T, Error>;
