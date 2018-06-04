extern crate rand;

use std::fs::*;
use std::path::PathBuf;
use rand::*;

#[derive(Debug)]
pub struct CleanTempDir {
    base_path: String,
    files: Vec<String>,
}

impl CleanTempDir {
    pub fn new() -> Self {
        create_dir_all("/tmp/whisper").expect("Pre-creating dir");
        CleanTempDir { base_path: String::from("/tmp/whisper"), files: vec![] }
    }

    pub fn get_file_path(&mut self, prefix: &str, extension: &str) -> PathBuf {
        let file_name = format!("{}/{}_{}.{}", self.base_path, prefix, random_string_suffix(), extension);
        self.files.push(file_name.clone());
        PathBuf::from(file_name)
    }
}

impl Drop for CleanTempDir {
    fn drop(&mut self) {
        self.files.iter().map(|f| remove_file(f).expect("remove file"));
    }
}

fn random_string_suffix() -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>()
}