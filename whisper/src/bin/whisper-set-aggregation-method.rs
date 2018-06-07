#[macro_use]
extern crate structopt;
extern crate failure;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use whisper::aggregation::AggregationMethod;
use whisper::set_aggregation_method;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-set-aggregation-method")]
struct Args {
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
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let old_aggregation_method = set_aggregation_method(
        &args.path,
        args.aggregation_method,
        Some(args.x_files_factor),
    )?;

    println!(
        "Updated aggregation method: {} ({} -> {})",
        &args.path.to_str().unwrap(),
        old_aggregation_method,
        &args.aggregation_method
    );

    Ok(())
}
