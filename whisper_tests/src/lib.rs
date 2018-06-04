use std::fs;
use std::panic::*;

pub const BENCH_FILE_PATH: &'static str = "/tmp/whisper";

pub struct CleanDir;

impl CleanDir {
    pub fn new() -> Self {
        fs::create_dir_all(BENCH_FILE_PATH).expect("Pre-creating dir");
        CleanDir
    }
}

impl Drop for CleanDir {
    fn drop(&mut self) {
         fs::remove_dir_all(BENCH_FILE_PATH).expect("Clean dir");
    }
}