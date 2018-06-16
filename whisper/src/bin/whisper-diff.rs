#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate serde_json;
extern crate whisper;

use failure::Error;
use std::fmt;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::diff;
use whisper::diff::DiffArchive;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-diff")]
struct Args {
    /// Path to data file
    #[structopt(name = "path_a", parse(from_os_str))]
    path_a: PathBuf,

    /// Path to data file
    #[structopt(name = "path_b", parse(from_os_str))]
    path_b: PathBuf,

    /// Show summary of differences
    #[structopt(long = "summary")]
    summary: bool,

    /// Skip comparison if either value is undefined
    #[structopt(long = "ignore-empty")]
    ignore_empty: bool,

    /// Print output in simple columns
    #[structopt(long = "columns")]
    columns: bool,

    /// Do not print column headers
    #[structopt(long = "no-headers")]
    no_headers: bool,

    /// Unix epoch time of the end of your requested
    #[structopt(long = "until")]
    until: Option<u32>,

    /// Output results in JSON form
    #[structopt(long = "json")]
    json: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiffArchiveContainer {
    #[serde(flatten)]
    inner: DiffArchive,
    points: usize,
}

impl From<DiffArchive> for DiffArchiveContainer {
    fn from(w: DiffArchive) -> DiffArchiveContainer {
        DiffArchiveContainer {
            points: w.diffs.len(),
            inner: w,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DiffArchiveSummary {
    archive: u32,
    total: u32,
    points: u32,
}

#[derive(Serialize, Deserialize)]
struct DiffArchiveJson {
    path_a: String,
    path_b: String,
    archives: Vec<DiffArchiveContainer>,
}

fn format_none(float: Option<f64>) -> String {
    match float {
        Some(x) => format!("{:.1}", x),
        None => format!("None"),
    }
}

impl fmt::Display for DiffArchiveJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for archive in &self.archives {
            if f.alternate() {
                writeln!(
                    f,
                    "Archive {} ({} of {} datapoints differ)",
                    archive.inner.index, archive.points, archive.inner.total
                )?;
                writeln!(
                    f,
                    "{:>7} {:>11} {:>13} {:>13}",
                    "", "timestamp", "value_a", "value_b"
                )?;
                for point in &archive.inner.diffs {
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
                for point in &archive.inner.diffs {
                    writeln!(
                        f,
                        "{} {} {} {}",
                        &archive.inner.index,
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

struct DiffHeader();
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
struct DiffArchiveSummaryJson {
    path_a: PathBuf,
    path_b: PathBuf,
    archives: Vec<DiffArchiveContainer>,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    for filename in &[&args.path_a, &args.path_b] {
        if !filename.is_file() {
            return Err(format_err!("[ERROR] File {:#?} does not exist!", filename));
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let until = args.until.unwrap_or(now);

    let diff_raw: Vec<DiffArchiveContainer> =
        diff::diff(&args.path_a, &args.path_b, args.ignore_empty, until, now)?
            .iter()
            .map(|x| (*x).to_owned().into())
            .collect();

    let diff_rich = DiffArchiveJson {
        path_a: args.path_a.display().to_string(),
        path_b: args.path_b.display().to_string(),
        archives: diff_raw,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&diff_rich)?);
    } else {
        if !args.columns {
            if !args.no_headers {
                println!("{:#}", DiffHeader {});
            }
            print!("{:#}", diff_rich);
        } else {
            if !args.no_headers {
                println!("{}", DiffHeader {});
            }
            print!("{}", diff_rich);
        }
    }

    Ok(())
}
