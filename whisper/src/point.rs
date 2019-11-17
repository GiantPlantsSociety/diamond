use crate::error::ParseError;
use async_std::io::{self, Read, Seek, Write};
use std::str::FromStr;

use crate::utils::{AsyncReadBytesExt, AsyncWriteBytesExt};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
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

    pub async fn read<R: Read + Seek + Unpin + Send>(read: &mut R) -> io::Result<Self> {
        let interval = read.aread_u32().await?;
        let value = read.aread_f64().await?;
        Ok(Self { interval, value })
    }

    pub async fn write<W: Write + Unpin + Send>(&self, write: &mut W) -> io::Result<()> {
        write.awrite_u32(self.interval).await?;
        write.awrite_f64(self.value).await?;
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
        assert_eq!(
            "1:2".parse(),
            Ok(Point {
                interval: 1,
                value: 2.0
            })
        );
        assert_eq!(
            "1:2.01".parse(),
            Ok(Point {
                interval: 1,
                value: 2.01
            })
        );
        assert_eq!(
            "1:0".parse(),
            Ok(Point {
                interval: 1,
                value: 0.0
            })
        );
        assert_eq!(
            "1:-2.01".parse(),
            Ok(Point {
                interval: 1,
                value: -2.01
            })
        );
        assert_eq!(
            "0:2".parse(),
            Ok(Point {
                interval: 0,
                value: 2.0
            })
        );
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
