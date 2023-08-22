use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::format_ts::display_ts;
use whisper::interval::Interval;
use whisper::WhisperFile;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Unix epoch time of the beginning of your requested interval (default: 24 hours ago)
    #[arg(long = "from")]
    from: Option<u32>,

    /// Unix epoch time of the end of your requested interval (default: now)
    #[arg(long = "until")]
    until: Option<u32>,

    /// Outputs results in JSON form
    #[arg(long = "json")]
    json: bool,

    /// Show human-readable timestamps instead of unix times
    #[arg(long = "pretty")]
    pretty: bool,

    /// Time format to use with --pretty; see https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html
    #[arg(long = "time-format", short = 't')]
    time_format: Option<String>,

    /// Specify 'nulls' to drop all null values. Specify 'zeroes' to drop all zero values. Specify 'empty' to drop both null and zero values
    #[arg(long = "drop")]
    drop: Option<String>,

    /// Path to data file
    #[arg(name = "path")]
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

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let from = args.from.unwrap_or(now - 86400);
    let until = args.until.unwrap_or(now);

    let interval = Interval::new(from, until)?;
    let mut file = WhisperFile::open(&args.path)?;

    let seconds_per_point = file
        .suggest_archive(interval, now)
        .ok_or_else(|| "No data in selected timerange")?;

    let filter = match args.drop {
        Some(ref s) if s == "nulls" => is_not_null,
        Some(ref s) if s == "zeroes" => is_not_zero,
        Some(ref s) if s == "empty" => is_not_empty,
        None => is_any,
        Some(ref s) => return Err(format!("No such drop option {}.", s).into()),
    };

    let archive = file
        .fetch(seconds_per_point, interval, now)?
        .filter_out(&filter);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&archive)?);
    } else {
        let time_format = match (&args.pretty, &args.time_format) {
            (true, Some(time_format)) => Some(time_format.as_str()),
            _ => None,
        };

        for (index, value) in archive.values.iter().enumerate() {
            let time = archive.from_interval + archive.step * index as u32;
            print!("{}\t", display_ts(i64::from(time), time_format));
            match value {
                Some(v) => println!("{}", v),
                None => println!("None"),
            }
        }
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
