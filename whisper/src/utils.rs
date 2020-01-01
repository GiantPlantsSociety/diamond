use async_trait::async_trait;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[async_trait]
pub trait AsyncReadBytesExt {
    async fn aread_f32(&mut self) -> io::Result<f32>;
    async fn aread_f64(&mut self) -> io::Result<f64>;
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send> AsyncReadBytesExt for R {
    async fn aread_f32(&mut self) -> io::Result<f32> {
        let mut f32_bytes = [0_u8; 4];
        self.read_exact(&mut f32_bytes).await?;
        std::io::Cursor::new(f32_bytes).read_f32::<BigEndian>()
    }

    async fn aread_f64(&mut self) -> io::Result<f64> {
        let mut f64_bytes = [0_u8; 8];
        self.read_exact(&mut f64_bytes).await?;
        std::io::Cursor::new(f64_bytes).read_f64::<BigEndian>()
    }
}

#[async_trait]
pub trait AsyncWriteBytesExt {
    async fn awrite_f32(&mut self, item: f32) -> io::Result<usize>;
    async fn awrite_f64(&mut self, item: f64) -> io::Result<usize>;
}

#[async_trait]
impl<R: AsyncWrite + Unpin + Send> AsyncWriteBytesExt for R {
    async fn awrite_f32(&mut self, item: f32) -> io::Result<usize> {
        let mut f32_bytes = vec![0_u8; 4];
        f32_bytes.write_f32::<BigEndian>(item)?;
        self.write(&f32_bytes).await
    }

    async fn awrite_f64(&mut self, item: f64) -> io::Result<usize> {
        let mut f64_bytes = vec![0_u8; 8];
        f64_bytes.write_f64::<BigEndian>(item)?;
        self.write(&f64_bytes).await
    }
}
