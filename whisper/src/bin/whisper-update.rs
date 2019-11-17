use async_std::task;
use failure::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::point::Point;
use whisper::WhisperFile;

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

async fn run(args: &Args) -> Result<(), Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let mut file = WhisperFile::open(&args.path).await?;
    file.update_many(&args.points, now).await?;
    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = task::block_on(run(&args)) {
        eprintln!("{}", err);
        exit(1);
    }
}
