use failure::Error;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs;
use std::path::PathBuf;
use tempfile::{Builder, TempDir};

use whisper::point::*;
use whisper::retention::*;
use whisper::*;

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

pub fn copy_test_file(temp_dir: &TempDir, filename: &str) -> PathBuf {
    let file_path = PathBuf::new().join("data").join(filename);

    let tmp_file_path = temp_dir.path().join(filename);

    fs::copy(&file_path, &tmp_file_path).unwrap();

    tmp_file_path
}

pub fn create_and_update_many(
    path: &PathBuf,
    timestamps: &[u32],
    now: u32,
) -> Result<WhisperFile, Error> {
    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .build(path)?;

    let points: Vec<Point> = timestamps
        .iter()
        .map(|interval| Point {
            interval: *interval,
            value: rand::random(),
        })
        .collect();

    file.update_many(&points, now)?;

    Ok(file)
}

pub fn create_and_update_points(
    path: &PathBuf,
    points: &[Point],
    now: u32,
) -> Result<WhisperFile, Error> {
    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .build(path)?;

    file.update_many(&points, now)?;

    Ok(file)
}
