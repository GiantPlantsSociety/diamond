#[macro_use]
extern crate structopt;
extern crate failure;
extern crate walkdir;
extern crate whisper;

use failure::Error;
use std::fs::remove_file;
use std::path::{PathBuf, Path};
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

fn collect_whisper_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(ref entry) if is_whisper_file(entry.path()) => Some(entry.path().to_path_buf()),
            _ => None
        })
        .collect::<Vec<PathBuf>>()
}

fn walk_dir(dir: &Path, delete_corrupt: bool) {
    let whisper_files = collect_whisper_files(dir);
    for file in &whisper_files {
        match whisper::WhisperFile::open(file) {
            Ok(whisper_file) => {
                let x: u32 = whisper_file.info().archives.iter().map(|a| a.points).sum();
                println!("{:?}: {} points", file, x);
            }
            _ => {
                if delete_corrupt {
                    eprintln!("Deleting corrupt Whisper file: {:?}", file);
                    remove_file(file).expect("Deleting File");
                } else {
                    eprintln!("Corrupt Whisper file: {:?}", file);
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    for dir in &args.directories {
        if !dir.is_dir() {
            eprintln!("{:?} is not a directory or not exist!", dir);
            exit(1);
        }

        if args.verbose {
            println!("Scanning {:?}...", dir);
        }

        walk_dir(dir, args.delete_corrupt);
    }

    Ok(())
}
