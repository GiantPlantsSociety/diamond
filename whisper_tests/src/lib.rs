extern crate rand;
extern crate tempfile;
extern crate assert_cli;


use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use tempfile::{TempDir, Builder};
use assert_cli::Assert;


pub fn get_temp_dir() -> TempDir {
    Builder::new()
        .prefix("whisper")
        .tempdir()
        .expect("Temp dir created")
}

pub fn get_file_path(temp_dir: &TempDir, prefix: &str) -> PathBuf {
    let file_name = format!("{}_{}.wsp", prefix, random_string(10));
    let mut path = temp_dir.path().to_path_buf();
    path.push(file_name);
    path
}

pub fn copy_test_file(temp_dir: &TempDir, filename: &str) -> PathBuf {
    let file_path = PathBuf::new()
        .join("data")
        .join(filename);

    let tmp_file_path = temp_dir.path()
        .join(filename);

    fs::copy(&file_path, &tmp_file_path).unwrap();

    tmp_file_path
}

pub fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect::<String>()
}

pub fn get_binary_command(binary_name: &str) -> Assert {
    Assert::command(&[
        OsStr::new("cargo"),
        OsStr::new("run"),
        #[cfg(not(debug_assertions))]
        OsStr::new("--release"),
        OsStr::new("--quiet"),
        OsStr::new("-p"),
        OsStr::new("whisper"),
        OsStr::new("--bin"),
        OsStr::new(binary_name),
        OsStr::new("--"),
    ])
}
