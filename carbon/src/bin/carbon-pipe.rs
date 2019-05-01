use failure::Error;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::aggregation::AggregationMethod;
use whisper::retention::Retention;
use carbon::line_update;

/// Receive metrics from pipe
#[derive(Debug, StructOpt)]
#[structopt(name = "carbon-pipe")]
struct Args {
    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

    /// XFILESFACTOR
    #[structopt(long = "xFilesFactor", default_value = "0.5")]
    x_files_factor: f32,

    /// Function to use when aggregating values
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[structopt(long = "aggregationMethod", default_value = "average")]
    aggregation_method: AggregationMethod,

    #[structopt(
        name = "retentions",
        help = r#"Specify lengths of time, for example:
60:1440      60 seconds per datapoint, 1440 datapoints = 1 day of retention
15m:8        15 minutes per datapoint, 8 datapoints = 2 hours of retention
1h:7d        1 hour per datapoint, 7 days of retention
12h:2y       12 hours per datapoint, 2 years of retention
"#,
        raw(required = "true", min_values = "1")
    )]
    retentions: Vec<Retention>,
}

fn run(args: &Args) -> Result<(), Error> {
    let stdin = io::stdin();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

    for line in stdin.lock().lines() {
        line_update(&line?, &args.path, now)?;
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
