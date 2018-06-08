#[macro_use]
extern crate structopt;
extern crate failure;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use whisper::WhisperBuilder;
use whisper::aggregation::AggregationMethod;
use whisper::retention::Retention;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-create")]
struct Args {
    /// Overwrite existing file
    #[structopt(long = "overwrite")]
    overwrite: bool,

    /// Don't create a whisper file, estimate storage requirements based on archive definitions
    #[structopt(long = "estimate")]
    estimate: bool,

    /// Create new whisper as sparse file
    #[structopt(long = "sparse")]
    sparse: bool,

    /// Create new whisper and use fallocate
    #[structopt(long = "fallocate")]
    fallocate: bool,

    /// XFILESFACTOR
    #[structopt(long = "xFilesFactor", default_value = "0.5")]
    x_files_factor: f32,

    /// Function to use when aggregating values
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[structopt(long = "aggregationMethod", default_value = "average")]
    aggregation_method: AggregationMethod,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

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

// whisper-create.py
// Usage: whisper-create.py path timePerPoint:timeToStore [timePerPoint:timeToStore]*
// whisper-create.py --estimate timePerPoint:timeToStore [timePerPoint:timeToStore]*

// timePerPoint and timeToStore specify lengths of time, for example:

// 60:1440      60 seconds per datapoint, 1440 datapoints = 1 day of retention
// 15m:8        15 minutes per datapoint, 8 datapoints = 2 hours of retention
// 1h:7d        1 hour per datapoint, 7 days of retention
// 12h:2y       12 hours per datapoint, 2 years of retention

// Options:
//   -h, --help            show this help message and exit
//   --xFilesFactor=XFILESFACTOR
//   --aggregationMethod=AGGREGATIONMETHOD
//                         Function to use when aggregating values (average, sum,
//                         last, max, min, avg_zero, absmax, absmin)
//   --overwrite
//   --estimate            Don't create a whisper file, estimate storage
//                         requirements based on archive definitions
//   --sparse              Create new whisper as sparse file
//   --fallocate           Create new whisper and use fallocate

// whisper-create.py load.1m.wsp 60:1440
// Created: load.1m.wsp (17308 bytes)

// whisper-create.py load.1m.wsp 60:1440 60:1440
// [ERROR] A Whisper database may not be configured having two archives with the same precision (archive0: (60, 1440), archive1: (60, 1440))

// whisper-create.py load.1m.wsp 60:1440 120:2880
// [ERROR] File load.1m.wsp already exists!

// whisper-create.py load.2m.wsp 60:1440 120:2880
// Created: load.2m.wsp (51880 bytes)

// whisper-create.py load.2m.wsp 60:1440 120:1440
// Created: load.2m.wsp (34600 bytes)

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    println!("whisper-create {}", env!("CARGO_PKG_VERSION"));
    println!("{:?}", args);

    let file = WhisperBuilder::default()
        .add_retentions(&args.retentions)
        .x_files_factor(args.x_files_factor)
        .aggregation_method(args.aggregation_method)
        .sparse(args.sparse)
        .build(args.path)?;

    println!("{:#?}", file.info());

    Ok(())
}
