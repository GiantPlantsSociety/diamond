use crate::point::Point;
use crate::POINT_SIZE;
use async_std::io::{self, Read, Seek, Write};
use async_std::prelude::*;

use crate::utils::{AsyncReadBytesExt, AsyncWriteBytesExt};

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

    pub async fn read<R: Read + Seek + Unpin + Send>(read: &mut R) -> io::Result<Self> {
        let offset = read.aread_u32().await?;
        let seconds_per_point = read.aread_u32().await?;
        let points = read.aread_u32().await?;
        Ok(Self {
            offset,
            seconds_per_point,
            points,
        })
    }

    pub async fn write<W: Write + Unpin + Send>(&self, write: &mut W) -> io::Result<()> {
        write.awrite_u32(self.offset).await?;
        write.awrite_u32(self.seconds_per_point).await?;
        write.awrite_u32(self.points).await?;
        Ok(())
    }

    pub async fn read_base<R: Read + Seek + Unpin + Send>(&self, r: &mut R) -> io::Result<Point> {
        r.seek(io::SeekFrom::Start(self.offset.into())).await?;
        Point::read(r).await
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
