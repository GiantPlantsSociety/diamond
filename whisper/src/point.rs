use std::io;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::str::FromStr;
use failure::Error;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub interval: u32,
    pub value: f64,
}

impl Point {
    pub fn align(&self, step: u32) -> Self {
        Self {
            interval: self.interval - (self.interval % step),
            value: self.value
        }
    }

    pub fn read<R: io::Read>(read: &mut R) -> Result<Self, io::Error> {
        let interval = read.read_u32::<BigEndian>()?;
        let value = read.read_f64::<BigEndian>()?;
        Ok(Self {
            interval,
            value,
        })
    }

    pub fn write<W: io::Write>(&self, write: &mut W) -> Result<(), io::Error> {
        write.write_u32::<BigEndian>(self.interval)?;
        write.write_f64::<BigEndian>(self.value)?;
        Ok(())
    }
}

impl FromStr for Point {
    type Err = Error;

    fn from_str(s: &str) -> Result<Point, Error> {
        let segments: Vec<&str> = s.split(":").collect();

        let (interval, value) = match segments.len() {
            2 => (segments[0], segments[1]),
            _ => return Err(format_err!("Cannot parse point from string: {}", s)),
        };

        Ok(Point{
            interval: interval.parse()?,
            value: value.parse()?,
        })
    }
}
