use std::path::Path;
use std::io;
use num::range_step;
use super::*;
use interval::Interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPoint {
    pub interval: u32,
    pub value1: Option<f64>,
    pub value2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffArchive {
    #[serde(rename = "archive")]
    pub index: usize,
    #[serde(rename = "points")]
    pub diffs: Vec<DiffPoint>,
    pub total: usize,
}

/** Compare two whisper databases. Each file must have the same archive configuration */
pub fn diff(path1: &Path, path2: &Path, ignore_empty: bool, mut until_time: u32, now: u32) -> Result<Vec<DiffArchive>, io::Error> {
    let mut file1 = WhisperFile::open(path1)?;
    let mut file2 = WhisperFile::open(path2)?;

    if file1.info().archives != file2.info().archives {
        return Err(io::Error::new(io::ErrorKind::Other, "Archive configurations are unalike. Resize the input before diffing"));
    }

    let mut archives = file1.info().archives.clone();
    archives.sort_by_key(|a| a.retention());

    let mut archive_diffs = Vec::new();

    for (index, archive) in archives.iter().enumerate() {
        let start_time = now - archive.retention();
        let interval = Interval::new(start_time, until_time).unwrap();

        let data1 = file1.fetch(archive.seconds_per_point, interval, now)?.unwrap();
        let data2 = file2.fetch(archive.seconds_per_point, interval, now)?.unwrap();

        let start = u32::min(data1.from_interval, data2.from_interval);
        let end = u32::max(data1.until_interval, data2.until_interval);
        let archive_step = u32::min(data1.step, data2.step);

        let points: Vec<DiffPoint> = range_step(start, end, archive_step)
            .enumerate()
            .map(|(index, interval)| DiffPoint {
                interval,
                value1: data1.values[index],
                value2: data2.values[index],
            })
            .filter(|p|
                if ignore_empty {
                    p.value1.is_some() && p.value2.is_some()
                } else {
                    p.value1.is_some() || p.value2.is_some()
                }
            )
            .collect();

        let total = points.len();

        let diffs = points.into_iter().filter(|p| p.value1 != p.value2).collect();

        archive_diffs.push(DiffArchive { index, diffs, total });

        until_time = u32::min(start_time, until_time);
    }

    Ok(archive_diffs)
}
