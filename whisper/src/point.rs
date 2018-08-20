use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use error::ParseError;
use std::io;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub interval: u32,
    pub value: f64,
}

impl Point {
    pub fn align(&self, step: u32) -> Self {
        Self {
            interval: self.interval - (self.interval % step),
            value: self.value,
        }
    }

    pub fn read<R: io::Read>(read: &mut R) -> Result<Self, io::Error> {
        let interval = read.read_u32::<BigEndian>()?;
        let value = read.read_f64::<BigEndian>()?;
        Ok(Self { interval, value })
    }

    pub fn write<W: io::Write>(&self, write: &mut W) -> Result<(), io::Error> {
        write.write_u32::<BigEndian>(self.interval)?;
        write.write_f64::<BigEndian>(self.value)?;
        Ok(())
    }
}

impl FromStr for Point {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Point, Self::Err> {
        let segments: Vec<&str> = s.split(':').collect();

        let (interval, value) = match segments.len() {
            2 => (segments[0], segments[1]),
            _ => return Err(ParseError::ParsePointError(s.to_string())),
        };

        Ok(Point {
            interval: interval.parse().map_err(ParseError::ParseIntError)?,
            value: value.parse().map_err(ParseError::ParseFloatError)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_parse_valid() {
        assert_eq!("1:2".parse(), Ok(Point{ interval: 1, value: 2.0}));
        assert_eq!("1:2.01".parse(), Ok(Point{ interval: 1, value: 2.01}));
        assert_eq!("1:0".parse(), Ok(Point{ interval: 1, value: 0.0}));
        assert_eq!("1:-2.01".parse(), Ok(Point{ interval: 1, value: -2.01}));
        assert_eq!("0:2".parse(), Ok(Point{ interval: 0, value: 2.0}));
    }

    #[test]
    fn point_parse_invalid() {
        assert!("-1:2.0".parse::<Point>().is_err());
        assert!("s:2.0".parse::<Point>().is_err());
        assert!("1:s".parse::<Point>().is_err());
        assert!("1111".parse::<Point>().is_err());
        assert!("ssss".parse::<Point>().is_err());
    }
}
