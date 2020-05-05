use diamond::line_update;
use diamond::settings::WhisperConfig;
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::aggregation::AggregationMethod;
use whisper::retention::Retention;

/// Receive metrics from pipe
#[derive(Debug, StructOpt)]
#[structopt(name = "diamond-pipe")]
struct Args {
    /// Path to the directory with data files
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

    /// Default value for the xFilesFactor for new files
    #[structopt(long = "xFilesFactor", default_value = "0.5")]
    x_files_factor: f32,

    /// Default function to use when aggregating values
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[structopt(long = "aggregationMethod", default_value = "average")]
    aggregation_method: AggregationMethod,

    #[structopt(
        name = "retentions",
        help = r#" Default retentions for new files
Specify lengths of time, for example:
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

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();

    let conf = WhisperConfig {
        retentions: args.retentions,
        aggregation_method: args.aggregation_method,
        x_files_factor: args.x_files_factor,
    };

    for line in stdin.lock().lines() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        line_update(&line?, &args.path, &conf, now)?;
    }
    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        exit(1);
    }
}
