#[macro_use]
extern crate structopt;
extern crate failure;
extern crate walkdir;
extern crate whisper;

use failure::Error;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "find-corrupt-whisper-files",
    about = "Find and (optionally) delete corrupt Whisper data files."
)]
struct Args {
    /// Delete reported files.
    #[structopt(long = "delete-corrupt", help = "Delete reported files")]
    delete_corrupt: bool,

    /// Display progress info.
    #[structopt(long = "verbose", help = "Display progress info")]
    verbose: bool,

    /// Directory containing Whisper files.
    #[structopt(
        name = "WHISPER_DIR",
        help = "Directory containing Whisper files",
        parse(from_os_str),
        raw(required = "true", min_values = "1")
    )]
    directories: Vec<PathBuf>,
}

fn is_whisper_file(path: &Path) -> bool {
    path.extension() == Some(std::ffi::OsStr::new("wsp"))
}

fn walk_dir(dir: &Path, delete_corrupt: bool, verbose: bool) -> Result<(), Error> {
    for entry in WalkDir::new(dir).min_depth(1).into_iter() {
        match entry {
            Ok(ref entry) if verbose && entry.file_type().is_dir() => {
                println!("Scanning {}...", entry.path().canonicalize()?.display())
            }
            Ok(ref entry) if is_whisper_file(entry.path()) => {
                delete_corrupt_file(&entry.path(), delete_corrupt)?
            }
            Err(e) => {
                eprintln!("{}", e)
            }
            _ => {}
        }
    }

    Ok(())
}

fn delete_corrupt_file(file: &Path, delete_corrupt: bool) -> Result<(), Error> {
    match whisper::WhisperFile::open(file) {
        Ok(whisper_file) => {
            let x: u32 = whisper_file.info().archives.iter().map(|a| a.points).sum();
            println!("{}: {} points", file.canonicalize()?.display(), x);
        }
        _ => {
            if delete_corrupt {
                eprintln!("Deleting corrupt Whisper file: {}", file.canonicalize()?.display());
                remove_file(file)?;
            } else {
                eprintln!("Corrupt Whisper file: {}", file.canonicalize()?.display());
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    for dir in &args.directories {
        if !dir.is_dir() {
            eprintln!("{} is not a directory or not exist!", dir.display());
            exit(1);
        }

        if args.verbose {
            println!("Scanning {}...", dir.canonicalize()?.display());
        }

        walk_dir(dir, args.delete_corrupt, args.verbose)?;
    }

    Ok(())
}
