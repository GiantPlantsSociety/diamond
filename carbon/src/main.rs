#[macro_use]
extern crate failure;
extern crate whisper;

use failure::Error;
use std::io::{self, BufRead};
use std::fs;
use std::path::PathBuf;
use whisper::*;


mod metrics;
use metrics::*;

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let dir = '.';

    for line in stdin.lock().lines() {
        let metric: MetricPoint = line?.parse()?;

        let metric_path: MetricPath = metric.name.parse()?;

        let file_path: PathBuf = metric_path.into();

        if !file_path.exists() {
            let dir_path = file_path.parent().unwrap();
            fs::create_dir_all(&dir_path)?;

            whisper::create(&meta, &file_path, true)?;
        }
        // println!("{}", line.unwrap());
    }
    Ok(())
}
