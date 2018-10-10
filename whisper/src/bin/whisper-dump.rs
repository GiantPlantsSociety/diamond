#[macro_use]
extern crate structopt;
extern crate chrono;
extern crate failure;
extern crate whisper;

use chrono::prelude::NaiveDateTime;
use failure::Error;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-dump")]
struct Args {
    /// Show human-readable timestamps instead of unix times
    #[structopt(long = "pretty", requires = "time_format")]
    pretty: bool,

    /// Time format to use with --pretty; see https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html
    #[structopt(long = "time-format", short = "t", requires = "pretty")]
    time_format: Option<String>,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

fn run(args: &Args) -> Result<(), Error> {
    let mut file = whisper::WhisperFile::open(&args.path)?;

    let meta = file.info().clone();
    println!("Meta data:");
    println!("  aggregation method: {}", &meta.aggregation_method);
    println!("  max retention: {}", &meta.max_retention);
    println!("  xFilesFactor: {}", &meta.x_files_factor);
    println!();

    for (i, archive) in meta.archives.iter().enumerate() {
        println!("Archive {} info:", i);
        println!("  offset: {}", &archive.offset);
        println!("  seconds per point: {}", &archive.seconds_per_point);
        println!("  points: {}", &archive.points);
        println!("  retention: {}", &archive.retention());
        println!("  size: {}", &archive.size());
        println!();
    }

    for (i, archive) in meta.archives.iter().enumerate() {
        let points = file.dump(archive.seconds_per_point)?;

        println!("Archive {} data:", i);
        for (j, point) in points.iter().enumerate() {
            match (&args.pretty, &args.time_format) {
                (true, Some(time_format)) => {
                    let timestr = NaiveDateTime::from_timestamp(i64::from(point.interval), 0)
                        .format(&time_format);
                    println!("{}: {}, {:>10}", j, timestr, &point.value);
                }
                (_, _) => {
                    println!("{}: {}, {:>10}", j, &point.interval, &point.value);
                }
            }
        }
        println!();
    }

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
