use std::fs::File;
use std::io::Result;

#[cfg(target_os = "unix")]
pub fn fallocate(fd: &mut File, offset: usize, len: usize) -> Result<()> {
    use std::os::ext::io::AsRawFd;

    libc::posix_fallocate(
        fd.as_raw_fd(),
        offset as ::libc::off_t,
        len as ::libc::off_t,
    );
}

#[cfg(not(target_os = "unix"))]
pub fn fallocate(fd: &mut File, offset: usize, len: usize) -> Result<()> {
    use std::io::{Seek, SeekFrom, Write};

    fd.seek(SeekFrom::Start(offset as u64))?;
    let zeroes = [0u8; 16384];
    let mut remaining = len;
    while remaining > zeroes.len() {
        fd.write_all(&zeroes)?;
        remaining -= zeroes.len();
    }
    fd.write_all(&zeroes[0..remaining])?;
    Ok(())
}
