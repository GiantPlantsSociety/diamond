extern crate carbon;
extern crate failure;
extern crate whisper;

use carbon::{MetricPath, MetricPoint};
use failure::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::builder::WhisperBuilder;
use whisper::retention::Retention;
use whisper::WhisperFile;

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

        let mut file = match file_path.exists() {
            false => {
                let dir_path = file_path.parent().unwrap();
                fs::create_dir_all(&dir_path)?;

                WhisperBuilder::default()
                    .add_retention(retention.clone())
                    .build(&file_path)
                    .unwrap()
            }
            true => WhisperFile::open(&file_path)?,
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        file.update(&metric.point, now)?;
    }
    Ok(())
}
