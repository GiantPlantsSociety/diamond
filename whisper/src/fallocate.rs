use std::fs::File;
use std::io::Result;

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
pub fn fallocate(fd: &mut File, offset: usize, len: usize) -> Result<()> {
    use std::os::unix::io::AsRawFd;

    unsafe {
        let ret = libc::posix_fallocate(
            fd.as_raw_fd(),
            offset as ::libc::off_t,
            len as ::libc::off_t,
        );

        if ret != 0 {
            return Err(std::io::Error::from_raw_os_error(ret));
        }
    }

    Ok(())
}

#[cfg(any(target_family = "windows", target_os = "macos"))]
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
