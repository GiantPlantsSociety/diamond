#[macro_use] extern crate structopt;
use structopt::StructOpt;
use std::process::exit;
use std::path::PathBuf;

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

    /// Time format to use with --pretty; see time.strftime()
    #[structopt(long = "time-format", short = "t")]
    time_format: Option<String>,

    /// Specify 'nulls' to drop all null values. Specify 'zeroes' to drop all zero values. Specify 'empty' to drop both null and zero values
    #[structopt(long = "drop")]
    drop: Option<String>,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

// whisper-fetch.py 
// Usage: whisper-fetch.py [options] path

// Options:
//   -h, --help            show this help message and exit
//   --from=_FROM          Unix epoch time of the beginning of your requested
//                         interval (default: 24 hours ago)
//   --until=UNTIL         Unix epoch time of the end of your requested interval
//                         (default: now)
//   --json                Output results in JSON form
//   --pretty              Show human-readable timestamps instead of unix times
//   -t TIME_FORMAT, --time-format=TIME_FORMAT
//                         Time format to use with --pretty; see time.strftime()
//   --drop=DROP           Specify 'nulls' to drop all null values. Specify
//                         'zeroes' to drop all zero values. Specify 'empty' to
//                         drop both null and zero values

fn run(args: &Args) -> Result<(), String> {
    println!("whisper-fetch {}", env!("CARGO_PKG_VERSION"));
    println!("{:?}", args);
    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
