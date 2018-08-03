use std::path::Path;
use std::io;
use super::*;
use interval::Interval;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPoint {
    #[serde(rename = "timestamp")]
    pub interval: u32,
    #[serde(rename = "value_a")]
    pub value1: Option<f64>,
    #[serde(rename = "value_b")]
    pub value2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffArchive {
    #[serde(rename = "archive")]
    pub index: usize,
    #[serde(rename = "datapoint")]
    pub diffs: Vec<DiffPoint>,
    pub points: usize,
    pub total: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DiffArchiveInfo {
    pub archives: Vec<DiffArchive>,
    pub path_a: String,
    pub path_b: String,
}

fn format_none(float: Option<f64>) -> String {
    match float {
        Some(x) => format!("{:.1}", x),
        None => "None".to_string(),
    }
}

impl fmt::Display for DiffArchiveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for archive in &self.archives {
            if f.alternate() {
                writeln!(
                    f,
                    "Archive {} ({} of {} datapoints differ)",
                    archive.index, archive.points, archive.total
                )?;
                writeln!(
                    f,
                    "{:>7} {:>11} {:>13} {:>13}",
                    "", "timestamp", "value_a", "value_b"
                )?;
                for point in &archive.diffs {
                    writeln!(
                        f,
                        "{:>7} {:>11} {:>13} {:>13}",
                        "",
                        point.interval,
                        format_none(point.value1),
                        format_none(point.value2)
                    )?;
                }
            } else {
                for point in &archive.diffs {
                    writeln!(
                        f,
                        "{} {} {} {}",
                        &archive.index,
                        point.interval,
                        format_none(point.value1),
                        format_none(point.value2)
                    )?;
                }
            }
        }

        Ok(())
    }
}

pub struct DiffHeader();

impl fmt::Display for DiffHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{:>7} {:>11} {:>13} {:>13}",
                "archive", "timestamp", "value_a", "value_b"
            )?;
        } else {
            write!(f, "archive timestamp value_a value_b")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffArchiveShort {
    #[serde(rename = "archive")]
    pub index: usize,
    pub total: usize,
    pub points: usize,
}

impl From<DiffArchive> for DiffArchiveShort {
    fn from(w: DiffArchive) -> DiffArchiveShort {
        DiffArchiveShort {
            index: w.index,
            total: w.total,
            points: w.points,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffArchiveSummary {
    pub archives: Vec<DiffArchiveShort>,
    pub path_a: String,
    pub path_b: String,
}

impl fmt::Display for DiffArchiveSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for archive in &self.archives {
            if f.alternate() {
                writeln!(
                    f,
                    "{:>7} {:>9} {:>9}",
                    archive.index, archive.total, archive.points
                )?;
            } else {
                writeln!(
                    f,
                    "{} {} {}",
                    archive.index, archive.total, archive.points
                )?;
            }
        }

        Ok(())
    }
}

pub struct DiffSummaryHeader();

impl fmt::Display for DiffSummaryHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:>7} {:>9} {:>9}", "archive", "total", "differing")?;
        } else {
            write!(f, "archive total differing")?;
        }
        Ok(())
    }
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

        let points: Vec<DiffPoint> = (start..end).step_by(archive_step as usize)
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

        let diffs: Vec<DiffPoint> = points.into_iter().filter(|p| p.value1 != p.value2).collect();
        let points = diffs.len();

        archive_diffs.push(DiffArchive { index, diffs, points, total });

        until_time = u32::min(start_time, until_time);
    }

    Ok(archive_diffs)
}
