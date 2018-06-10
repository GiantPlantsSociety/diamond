#[macro_use]
extern crate structopt;
extern crate failure;
extern crate whisper;

use failure::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use whisper::aggregation::AggregationMethod;
use whisper::retention::Retention;
use whisper::WhisperBuilder;

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

    /// Create new whisper and use fallocate, default behavior, left for compatibility
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

fn byte_format(number: usize) -> String {
    let units = ["bytes", "KB", "MB"];
    let mut size = number as f64;
    let mut unit = "GB";

    for u in units.iter() {
        if size < 1024.0 {
            unit = u;
            break;
        }
        size = size / 1024.0;
    }
    format!("{:.3}{}", size, unit)
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    if args.estimate {
        for (i, retention) in args.retentions.iter().enumerate() {
            println!(
                "Archive {}: {} points of {}s precision",
                i, &retention.points, &retention.seconds_per_point
            );
        }

        let total_points: usize = args.retentions.iter().map(|x| x.points as usize).sum();

        let size = (whisper::METADATA_SIZE + (args.retentions.len() * whisper::ARCHIVE_INFO_SIZE)
            + (total_points * whisper::POINT_SIZE)) as usize;
        let disk_size = (size as f64 / 4096.0).ceil() as usize * 4096;

        println!();
        println!(
            "Estimated Whisper DB Size: {} ({} bytes on disk with 4k blocks)",
            byte_format(size),
            disk_size
        );
        println!();

        let numbers = [1, 5, 10, 50, 100, 500];

        for number in numbers.iter() {
            println!(
                "Estimated storage requirement for {}k metrics: {}",
                number,
                byte_format(number * 1000_usize * disk_size)
            );
        }
    } else {
        if args.overwrite && args.path.exists() {
            println!(
                "Overwriting existing file: {}",
                &args.path.to_str().unwrap()
            );
            fs::remove_file(&args.path)?;
        }

        WhisperBuilder::default()
            .add_retentions(&args.retentions)
            .x_files_factor(args.x_files_factor)
            .aggregation_method(args.aggregation_method)
            .sparse(args.sparse)
            .build(&args.path)?;

        let size = args.path.metadata()?.len();
        println!("Created: {} ({} bytes)", &args.path.to_str().unwrap(), size);
    }

    Ok(())
}
