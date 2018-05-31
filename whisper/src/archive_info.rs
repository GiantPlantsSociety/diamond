use std::io;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use point::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArchiveInfo {
    pub offset: u32,
    pub seconds_per_point: u32,
    pub points: u32,
    // 'size': points * pointSize,
}

impl ArchiveInfo {
    pub fn retention(&self) -> u32 {
        self.seconds_per_point * self.points
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
