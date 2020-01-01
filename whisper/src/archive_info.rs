use crate::point::Point;
use crate::POINT_SIZE;
use std::io::SeekFrom;
use tokio::io::{
    self, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt,
};

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

    pub async fn read<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, io::Error> {
        let offset = read.read_u32().await?;
        let seconds_per_point = read.read_u32().await?;
        let points = read.read_u32().await?;
        Ok(Self {
            offset,
            seconds_per_point,
            points,
        })
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), io::Error> {
        write.write_u32(self.offset).await?;
        write.write_u32(self.seconds_per_point).await?;
        write.write_u32(self.points).await
    }

    pub async fn read_base<R: AsyncRead + AsyncSeek + Unpin + Send>(
        &self,
        r: &mut R,
    ) -> Result<Point, io::Error> {
        r.seek(SeekFrom::Start(self.offset.into())).await?;
        let base = Point::read(r).await?;
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
