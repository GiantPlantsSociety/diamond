use crate::POINT_SIZE;
use crate::point::Point;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArchiveInfo {
    pub offset: u32,
    pub seconds_per_point: u32,
    pub points: u32,
}

impl ArchiveInfo {
    pub fn retention(&self) -> u32 {
        self.seconds_per_point * self.points
    }

    pub fn size(&self) -> usize {
        self.points as usize * POINT_SIZE
    }

    pub fn read<R: io::Read>(read: &mut R) -> Result<Self, io::Error> {
        let offset = read.read_u32::<BigEndian>()?;
        let seconds_per_point = read.read_u32::<BigEndian>()?;
        let points = read.read_u32::<BigEndian>()?;
        Ok(Self {
            offset,
            seconds_per_point,
            points,
        })
    }

    pub fn write<W: io::Write>(&self, write: &mut W) -> Result<(), io::Error> {
        write.write_u32::<BigEndian>(self.offset)?;
        write.write_u32::<BigEndian>(self.seconds_per_point)?;
        write.write_u32::<BigEndian>(self.points)?;
        Ok(())
    }

    pub fn read_base<R: io::Read + io::Seek>(&self, r: &mut R) -> Result<Point, io::Error> {
        r.seek(io::SeekFrom::Start(self.offset.into()))?;
        let base = Point::read(r)?;
        Ok(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention() {
        let info = ArchiveInfo {
            offset: 10,
            seconds_per_point: 2,
            points: 20,
        };
        assert_eq!(info.retention(), 20 * 2);
    }

    #[test]
    fn test_size() {
        let info = ArchiveInfo {
            offset: 10,
            seconds_per_point: 2,
            points: 20,
        };
        assert_eq!(info.size(), 20 * 12);
    }
}
