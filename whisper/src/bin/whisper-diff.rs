#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate serde_json;
extern crate whisper;

use failure::Error;
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

#[derive(Debug, Serialize, Deserialize)]
struct DiffArchiveJson {
    path_a: String,
    path_b: String,
    archives: Vec<DiffArchiveContainer>,
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
        if !filename.exists() {
            eprintln!(
                "[ERROR] File \"{}\" does not exist!",
                filename.to_str().unwrap()
            );
            exit(1);
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let until = args.until.unwrap_or(now);

    let diff: Vec<DiffArchiveContainer> =
        diff::diff(&args.path_a, &args.path_b, args.ignore_empty, until, now)?
            .iter()
            .map(|x| (*x).to_owned().into())
            .collect();

    let diff_json = DiffArchiveJson {
        path_a: args.path_a.clone().to_str().unwrap().to_owned(),
        path_b: args.path_b.clone().to_str().unwrap().to_owned(),
        archives: diff,
    };

    println!("{}", serde_json::to_string_pretty(&diff_json)?);

    Ok(())
}
