use chrono::prelude::NaiveDateTime;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-dump")]
struct Args {
    /// Time format to show human-readable time instead of unix timestamp; see https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html
    #[structopt(long = "time-format", short = "t")]
    time_format: Option<String>,

    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

fn run(args: &Args) -> io::Result<()> {
    let mut file = whisper::WhisperFile::open(&args.path)?;

    let meta = file.info().clone();
    println!("Meta data:");
    println!("  aggregation method: {}", &meta.aggregation_method);
    println!("  max retention: {}", &meta.max_retention);
    println!("  xFilesFactor: {}", &meta.x_files_factor);
    println!();

    for (i, archive) in meta.archives.iter().enumerate() {
        println!("Archive {} info:", i);
        println!("  offset: {}", &archive.offset);
        println!("  seconds per point: {}", &archive.seconds_per_point);
        println!("  points: {}", &archive.points);
        println!("  retention: {}", &archive.retention());
        println!("  size: {}", &archive.size());
        println!();
    }

    for (i, archive) in meta.archives.iter().enumerate() {
        let points = file.dump(archive.seconds_per_point)?;

        println!("Archive {} data:", i);
        for (j, point) in points.iter().enumerate() {
            match args.time_format {
                Some(ref time_format) => {
                    let timestr = NaiveDateTime::from_timestamp(i64::from(point.interval), 0)
                        .format(&time_format);
                    println!("{}: {}, {:>10}", j, timestr, &point.value);
                }
                _ => {
                    println!("{}: {}, {:>10}", j, &point.interval, &point.value);
                }
            }
        }
        println!();
    }

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
