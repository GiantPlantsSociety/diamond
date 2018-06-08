extern crate tempfile;

use self::tempfile::{TempDir, Builder};
use std::path::PathBuf;
use std::fs;

pub fn get_temp_dir() -> TempDir {
    Builder::new()
        .prefix("whisper")
        .tempdir()
        .expect("Temp dir created")
}

pub fn copy_test_file(temp_dir: &TempDir, filename: &str) -> PathBuf {
    let file_path = PathBuf::new()
        .join("tests")
        .join("data")
        .join(filename);

    let tmp_file_path = temp_dir.path()
        .join(filename);

    fs::copy(&file_path, &tmp_file_path).unwrap();

    tmp_file_path
}
