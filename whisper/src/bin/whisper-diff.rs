#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate serde_json;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::diff;
use whisper::diff::DiffArchive;
use whisper::diff::DiffArchiveInfo;
use whisper::diff::DiffArchiveShort;
use whisper::diff::DiffArchiveSummary;
use whisper::diff::DiffHeader;
use whisper::diff::DiffSummaryHeader;

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

fn print_details(
    diff: &DiffArchiveInfo,
    json: bool,
    columns: bool,
    no_headers: bool,
) -> Result<(), Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(diff)?);
    } else if !columns {
        if !no_headers {
            println!("{:#}", DiffHeader {});
        }
        print!("{:#}", diff);
    } else {
        if !no_headers {
            println!("{}", DiffHeader {});
        }
        print!("{}", diff);
    }

    Ok(())
}

fn print_summary(
    diff: &DiffArchiveSummary,
    json: bool,
    columns: bool,
    no_headers: bool,
) -> Result<(), Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(diff)?);
    } else if !columns {
        if !no_headers {
            println!("{:#}", DiffSummaryHeader {});
        }
        print!("{:#}", diff);
    } else {
        if !no_headers {
            println!("{}", DiffSummaryHeader {});
        }
        print!("{}", diff);
    }

    Ok(())
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

    let diff_raw = diff::diff(&args.path_a, &args.path_b, args.ignore_empty, until, now)?;

    if args.summary {
        let short_diff: Vec<DiffArchiveShort> =
            diff_raw.iter().map(|x| (*x).to_owned().into()).collect();

        let diff_rich = DiffArchiveSummary {
            path_a: args.path_a.display().to_string(),
            path_b: args.path_b.display().to_string(),
            archives: short_diff,
        };
        print_summary(&diff_rich, args.json, args.columns, args.no_headers)?;
    } else {
        let diff_rich = DiffArchiveInfo {
            path_a: args.path_a.display().to_string(),
            path_b: args.path_b.display().to_string(),
            archives: diff_raw,
        };
        print_details(&diff_rich, args.json, args.columns, args.no_headers)?;
    }
    Ok(())
}
