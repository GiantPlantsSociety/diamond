use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::WhisperFile;
use whisper::point::Point;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to data file
    #[arg(name = "path")]
    path: PathBuf,

    /// Set of data points
    #[arg(name = "timestamp:value")]
    points: Vec<Point>,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

    let mut file = WhisperFile::open(&args.path)?;
    file.update_many(&args.points, now)?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
