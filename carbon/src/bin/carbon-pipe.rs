extern crate carbon;
extern crate failure;
extern crate whisper;

use carbon::{MetricPath, MetricPoint};
use failure::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use whisper::builder::WhisperBuilder;
use whisper::retention::Retention;

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let _dir = '.';

    let retention = Retention {
        seconds_per_point: 60,
        points: 10,
    };

    for line in stdin.lock().lines() {
        let metric: MetricPoint = line?.parse()?;
        let metric_path: MetricPath = metric.name.parse()?;

        let file_path: PathBuf = metric_path.into();

        if !file_path.exists() {
            let dir_path = file_path.parent().unwrap();
            fs::create_dir_all(&dir_path)?;

            let mut file = WhisperBuilder::default()
                .add_retention(retention.clone())
                .build(&file_path)
                .unwrap();
        }
    }
    Ok(())
}
