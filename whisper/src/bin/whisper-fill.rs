use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::fill::fill;

/// Copies data from src to dst, if missing.
#[derive(Debug, clap::Parser)]
struct Args {
    /// Lock whisper files (is not implemented).
    #[arg(long = "lock")]
    lock: bool,

    /// Source whisper file.
    #[arg(name = "SRC")]
    src: PathBuf,

    /// Destination whisper file.
    #[arg(name = "DST")]
    dst: PathBuf,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    fill(&args.src, &args.dst, now, now)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
