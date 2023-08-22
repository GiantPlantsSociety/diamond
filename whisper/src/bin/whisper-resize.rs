use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::aggregation::AggregationMethod;
use whisper::error;
use whisper::resize::resize;
use whisper::retention::Retention;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Change the xFilesFactor
    #[arg(long = "xFilesFactor")]
    x_files_factor: Option<f32>,

    /// Change the aggregation function:
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[arg(long = "aggregationMethod")]
    aggregation_method: Option<AggregationMethod>,

    /// Perform a destructive change
    #[arg(long = "force")]
    force: bool,

    /// Create a new database file without removing the existing one
    #[arg(long = "newfile")]
    newfile: Option<PathBuf>,

    /// Delete the .bak file after successful execution
    #[arg(long = "nobackup")]
    nobackup: bool,

    /// Try to aggregate the values to fit the new archive better.
    /// Note that this will make things slower and use more memory.
    #[arg(long = "aggregate")]
    aggregate: bool,

    /// Path to data file
    #[arg(name = "path")]
    path: PathBuf,

    #[arg(
        name = "retentions",
        help = r#"timePerPoint and timeToStore specify lengths of time, for example:
60:1440      60 seconds per datapoint, 1440 datapoints = 1 day of retention
15m:8        15 minutes per datapoint, 8 datapoints = 2 hours of retention
1h:7d        1 hour per datapoint, 7 days of retention
12h:2y       12 hours per datapoint, 2 years of retention
"#,
        required = true
    )]
    retentions: Vec<Retention>,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let path = &args.path;

    if !path.is_file() {
        return Err(error::Error::FileNotExist(path.to_path_buf()).into());
    }

    let whisper_file = whisper::WhisperFile::open(path)?;
    let meta = whisper_file.info();

    let x_files_factor = args.x_files_factor.unwrap_or(meta.x_files_factor);
    let aggregation_method = args.aggregation_method.unwrap_or(meta.aggregation_method);

    println!("Retrieving all data from the archives");

    resize(
        path,
        args.newfile.as_deref(),
        &args.retentions,
        x_files_factor,
        aggregation_method,
        args.aggregate,
        args.nobackup,
        now,
    )?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
