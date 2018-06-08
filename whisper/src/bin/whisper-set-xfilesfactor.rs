#[macro_use]
extern crate structopt;
extern crate failure;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "whisper-set-xfilesfactor",
    about = "Set xFilesFactor for existing whisper files"
)]
struct Args {
    /// Path to data file
    #[structopt(name = "path", parse(from_os_str), help = "path to whisper file")]
    path: PathBuf,

    /// XFILESFACTOR
    #[structopt(name = "xFilesFactor", help = "new xFilesFactor, a float between 0 and 1")]
    x_files_factor: f32,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let mut file = whisper::WhisperFile::open(&args.path)?;

    let old_x_files_factor = file.info().x_files_factor;

    file.set_x_files_factor(args.x_files_factor)?;

    println!(
        "Updated xFilesFactor: {} ({} -> {})",
        &args.path.to_str().unwrap(),
        old_x_files_factor,
        &args.x_files_factor
    );

    Ok(())
}
