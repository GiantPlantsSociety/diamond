#[macro_use]
extern crate structopt;
extern crate failure;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::point::Point;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-update")]
struct Args {
    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,

    /// Set of data points
    #[structopt(name = "timestamp:value")]
    points: Vec<Point>,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    whisper::update_many(&args.path, &args.points, now)?;
    Ok(())
}
