use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::diff;
use whisper::diff::{
    DiffArchiveInfo, DiffArchiveShort, DiffArchiveSummary, DiffHeader, DiffSummaryHeader,
};

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to data file
    #[arg(name = "path_a")]
    path_a: PathBuf,

    /// Path to data file
    #[arg(name = "path_b")]
    path_b: PathBuf,

    /// Show summary of differences
    #[arg(long = "summary")]
    summary: bool,

    /// Skip comparison if either value is undefined
    #[arg(long = "ignore-empty")]
    ignore_empty: bool,

    /// Print output in simple columns
    #[arg(long = "columns")]
    columns: bool,

    /// Do not print column headers
    #[arg(long = "no-headers")]
    no_headers: bool,

    /// Unix epoch time of the end of your requested
    #[arg(long = "until")]
    until: Option<u32>,

    /// Output results in JSON form
    #[arg(long = "json")]
    json: bool,
}

fn print_details(
    diff: &DiffArchiveInfo,
    json: bool,
    columns: bool,
    no_headers: bool,
) -> Result<(), Box<dyn Error>> {
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
) -> Result<(), Box<dyn Error>> {
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

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    for filename in &[&args.path_a, &args.path_b] {
        if !filename.is_file() {
            return Err(format!("[ERROR] File {:#?} does not exist!", filename).into());
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

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
