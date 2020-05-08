use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::merge::merge;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-merge")]
struct Args {
    /// Path to data file
    #[structopt(name = "from_path", parse(from_os_str))]
    from_path: PathBuf,

    /// Path to data file
    #[structopt(name = "to_path", parse(from_os_str))]
    to_path: PathBuf,

    /// Begining of interval, unix timestamp(default: epoch)
    #[structopt(long = "from")]
    from: Option<u32>,

    /// End of interval, unix timestamp (default: now)
    #[structopt(long = "until")]
    until: Option<u32>,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    for filename in &[&args.from_path, &args.to_path] {
        if !filename.is_file() {
            return Err(format!("[ERROR] File \"{:?}\" does not exist!", filename).into());
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let from = args.from.unwrap_or(0);
    let until = args.until.unwrap_or(now);

    merge(&args.from_path, &args.to_path, from, until, now)?;

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
