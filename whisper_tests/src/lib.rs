extern crate rand;
extern crate tempfile;

use std::path::PathBuf;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use tempfile::{TempDir, Builder};

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

pub fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect::<String>()
}
