#[macro_use]
extern crate structopt;
extern crate chrono;
extern crate failure;
extern crate whisper;

use chrono::prelude::NaiveDateTime;
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use whisper::{info, read_archive_all};

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-dump")]
struct Args {
    /// Show human-readable timestamps instead of unix times
    #[structopt(long = "pretty", requires = "time_format")]
    pretty: bool,

    /// Time format to use with --pretty; see https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html
    #[structopt(long = "time-format", short = "t", requires = "pretty")]
    time_format: Option<String>,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let meta = info(&args.path)?;
    println!("Meta data:");
    println!("  aggregation method: {}", &meta.aggregation_method);
    println!("  max retention: {}", &meta.max_retention);
    println!("  xFilesFactor: {}", &meta.x_files_factor);
    println!();

    for (i, archive) in meta.archives.iter().enumerate() {
        println!("Archive {} info:", i);
        println!("  offset: {}", &archive.offset,);
        println!("  seconds per point: {}", &archive.seconds_per_point);
        println!("  points: {}", &archive.points);
        println!("  retention: {}", &archive.retention());
        println!("  size: {}", &archive.size());
        println!();

        let points = read_archive_all(&args.path, archive)?;

        println!("Archive {} data:", i);
        for (j, point) in points.iter().enumerate() {
            match (&args.pretty, &args.time_format) {
                (true, Some(time_format)) => {
                    let timestr = NaiveDateTime::from_timestamp(point.interval as i64, 0)
                        .format(&time_format);
                    println!("{}: {}, {}", j, timestr, &point.value);
                }
                (_, _) => {
                    println!("{}: {}, {}", j, &point.interval, &point.value);
                }
            }
        }
    }

    Ok(())
}
