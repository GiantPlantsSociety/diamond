#[macro_use]
extern crate structopt;
extern crate chrono;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate whisper;

use chrono::prelude::NaiveDateTime;
use failure::{err_msg, Error};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

use whisper::interval::Interval;
use whisper::WhisperFile;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-fetch")]
struct Args {
    /// Unix epoch time of the beginning of your requested interval (default: 24 hours ago)
    #[structopt(long = "from")]
    from: Option<u32>,

    /// Unix epoch time of the end of your requested interval (default: now)
    #[structopt(long = "until")]
    until: Option<u32>,

    /// Outputs results in JSON form
    #[structopt(long = "json")]
    json: bool,

    /// Show human-readable timestamps instead of unix times
    #[structopt(long = "pretty")]
    pretty: bool,

    /// Time format to use with --pretty; see https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html
    #[structopt(long = "time-format", short = "t")]
    time_format: Option<String>,

    /// Specify 'nulls' to drop all null values. Specify 'zeroes' to drop all zero values. Specify 'empty' to drop both null and zero values
    #[structopt(long = "drop")]
    drop: Option<String>,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

fn is_not_zero(value: &Option<f64>) -> bool {
    value != &Some(0_f64)
}

fn is_not_null(value: &Option<f64>) -> bool {
    value.is_some()
}

fn is_not_empty(value: &Option<f64>) -> bool {
    is_not_null(value) && is_not_zero(value)
}

fn is_any(_value: &Option<f64>) -> bool {
    true
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let from = args.from.unwrap_or(now - 86400);
    let until = args.until.unwrap_or(now);

    let interval = Interval::new(from, until).map_err(err_msg)?;
    let mut file = WhisperFile::open(&args.path)?;

    let seconds_per_point = whisper::suggest_archive(&file, interval, now)
        .ok_or(err_msg("No data in selected timerange"))?;

    let filter = match args.drop {
        Some(ref s) if s == "nulls" => is_not_null,
        Some(ref s) if s == "zeroes" => is_not_zero,
        Some(ref s) if s == "empty" => is_not_empty,
        None => is_any,
        Some(ref s) => return Err(format_err!("No such drop option {}.", s)),
    };

    let archive = file.fetch(seconds_per_point, interval, now)?
        .ok_or(err_msg("No data in selected timerange"))?
        .filter_out(&filter);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&archive)?);
    } else {
        for (index, value) in archive.values.iter().enumerate() {
            let time = archive.from_interval + archive.step * index as u32;

            match (&args.pretty, &args.time_format) {
                (true, Some(time_format)) => {
                    let timestr =
                        NaiveDateTime::from_timestamp(i64::from(time), 0).format(&time_format);
                    println!(
                        "{}\t{}",
                        timestr,
                        value.map(|x| x.to_string()).unwrap_or("None".to_owned())
                    );
                }
                (_, _) => println!(
                    "{}\t{}",
                    time,
                    value.map(|x| x.to_string()).unwrap_or("None".to_owned())
                ),
            }
        }
    }

    Ok(())
}
