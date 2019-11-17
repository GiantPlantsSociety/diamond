use failure::Error;

use async_std::task;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

use whisper::aggregation::AggregationMethod;
use whisper::error;
use whisper::resize::resize;
use whisper::retention::Retention;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-resize")]
struct Args {
    /// Change the xFilesFactor
    #[structopt(long = "xFilesFactor")]
    x_files_factor: Option<f32>,

    /// Change the aggregation function:
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[structopt(long = "aggregationMethod")]
    aggregation_method: Option<AggregationMethod>,

    /// Perform a destructive change
    #[structopt(long = "force")]
    force: bool,

    /// Create a new database file without removing the existing one
    #[structopt(long = "newfile", parse(from_os_str))]
    newfile: Option<PathBuf>,

    /// Delete the .bak file after successful execution
    #[structopt(long = "nobackup")]
    nobackup: bool,

    /// Try to aggregate the values to fit the new archive better.
    /// Note that this will make things slower and use more memory.
    #[structopt(long = "aggregate")]
    aggregate: bool,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

    #[structopt(
        name = "retentions",
        help = r#"timePerPoint and timeToStore specify lengths of time, for example:
60:1440      60 seconds per datapoint, 1440 datapoints = 1 day of retention
15m:8        15 minutes per datapoint, 8 datapoints = 2 hours of retention
1h:7d        1 hour per datapoint, 7 days of retention
12h:2y       12 hours per datapoint, 2 years of retention
"#,
        required = true,
        min_values = 1
    )]
    retentions: Vec<Retention>,
}

async fn run(args: &Args) -> Result<(), Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let path = &args.path;

    if !path.is_file() {
        return Err(error::Error::FileNotExist(path.to_path_buf()).into());
    }

    let whisper_file = whisper::WhisperFile::open(path).await?;
    let meta = whisper_file.info();

    let x_files_factor = args.x_files_factor.unwrap_or(meta.x_files_factor);
    let aggregation_method = args.aggregation_method.unwrap_or(meta.aggregation_method);

    println!("Retrieving all data from the archives");

    resize(
        path,
        args.newfile.as_ref().map(PathBuf::as_path),
        &args.retentions,
        x_files_factor,
        aggregation_method,
        args.aggregate,
        args.nobackup,
        now,
    )
    .await?;

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = task::block_on(run(&args)) {
        eprintln!("{}", err);
        exit(1);
    }
}
