use std::io;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use whisper::aggregation::AggregationMethod;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-set-aggregation-method")]
struct Args {
    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

    /// Function to use when aggregating values
    /// (average, sum, last, max, min, avg_zero, absmax, absmin)
    #[structopt(name = "aggregationMethod", default_value = "average")]
    aggregation_method: AggregationMethod,

    /// XFILESFACTOR
    #[structopt(name = "xFilesFactor", default_value = "0.5")]
    x_files_factor: f32,
}

fn run(args: &Args) -> io::Result<()> {
    let mut file = whisper::WhisperFile::open(&args.path)?;

    let old_aggregation_method = file.info().aggregation_method;

    file.set_x_files_factor(args.x_files_factor)?;
    file.set_aggregation_method(args.aggregation_method)?;

    println!(
        "Updated aggregation method: {} ({} -> {})",
        &args.path.to_str().unwrap(),
        old_aggregation_method,
        &args.aggregation_method
    );

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
