use crate::aggregation::AggregationMethod;
use crate::builder::WhisperBuilder;
use crate::error::Error;
use crate::interval::Interval;

use crate::WhisperFile;
use crate::point::Point;
use crate::retention::Retention;

use std::fs::{remove_file, rename};
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;

fn migrate_aggregate(path_src: &Path, path_dst: &Path, now: u32) -> io::Result<()> {
    let mut file_src = WhisperFile::open(path_src)?;
    let mut file_dst = WhisperFile::open(path_dst)?;

    let meta = file_src.info().clone();
    let mut until = now;

    for archive in &meta.archives {
        let interval =
            Interval::new(0, until).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let (adjusted_interval, data) =
            file_src.fetch_points(archive.seconds_per_point, interval, now)?;

        if let Some(ref data) = data {
            let mut points_to_write: Vec<Point> = data
                .clone()
                .into_iter()
                .filter(|point| point != &Point::default())
                .collect();

            points_to_write.sort_by_key(|point| point.interval);

            println!(
                "({},{},{})",
                adjusted_interval.from(),
                adjusted_interval.until(),
                archive.seconds_per_point
            );

            let values = points_to_write
                .iter()
                .map(|point| point.interval.to_string())
                .collect::<Vec<String>>()
                .join(" ");

            println!("timepoints_to_update: {}", values);

            until = points_to_write.get(0).map(|x| x.interval).unwrap_or(now);
            file_dst.update_many(&points_to_write, now)?;
        }
    }

    Ok(())
}

fn migrate_nonaggregate(path_src: &Path, path_dst: &Path, now: u32) -> io::Result<()> {
    let mut file_src = WhisperFile::open(path_src)?;
    let mut file_dst = WhisperFile::open(path_dst)?;

    let interval = Interval::new(0, now).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let meta = file_src.info().clone();
    let mut archives = meta.archives;
    archives.sort_by_key(|archive| archive.retention());

    for archive in &archives {
        let (_adjusted_interval, data) =
            file_src.fetch_points(archive.seconds_per_point, interval, now)?;

        if let Some(ref data) = data {
            let mut points_to_write: Vec<Point> = data
                .clone()
                .into_iter()
                .filter(|point| point != &Point::default())
                .collect();

            points_to_write.sort_by_key(|point| point.interval);
            file_dst.update_many(&points_to_write, now)?;
        }
    }

    Ok(())
}

fn migrate_points(
    path_src: &Path,
    path_dst: &Path,
    aggregate: bool,
    now: u32,
) -> Result<(), Error> {
    if !path_src.is_file() {
        return Err(Error::FileNotExist(path_src.to_owned()).into());
    }

    if aggregate {
        println!("Migrating data with aggregation...");
        migrate_aggregate(path_src, path_dst, now)?;
    } else {
        println!("Migrating data without aggregation...");
        migrate_nonaggregate(path_src, path_dst, now)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn resize(
    path_src: &Path,
    path_new: Option<&Path>,
    retentions: &[Retention],
    x_files_factor: f32,
    aggregation_method: AggregationMethod,
    aggregate: bool,
    nobackup: bool,
    now: u32,
) -> Result<(), Error> {
    let path_dst = match path_new {
        None => {
            let tmpfile = PathBuf::from(format!("{}.tmp", path_src.display()));
            if tmpfile.is_file() {
                println!(
                    "Removing previous temporary database file: {}",
                    tmpfile.display()
                );
                remove_file(&tmpfile)?;
            }
            tmpfile
        }
        Some(new) => new.to_path_buf(),
    };

    println!("Retrieving all data from the archives");
    println!("Creating new whisper database: {}", path_dst.display());

    WhisperBuilder::default()
        .add_retentions(retentions)
        .x_files_factor(x_files_factor)
        .aggregation_method(aggregation_method)
        .build(&path_dst)?;

    let size = path_dst.metadata()?.len();
    println!("Created: {} ({} bytes)", path_dst.display(), size);

    migrate_points(path_src, &path_dst, aggregate, now)?;

    if path_new.is_some() {
        return Ok(());
    }

    let backup = format!("{}.bak", path_src.display());
    println!("Renaming old database to: {}", backup);
    rename(path_src, &backup)?;

    println!("Renaming new database to: {}", path_src.display());
    rename(&path_dst, path_src)
        .map_err(|e| {
            eprintln!("{}", e);
            println!("Operation failed, restoring backup");
            rename(&backup, path_src).unwrap();
            exit(1);
        })
        .unwrap();

    if nobackup {
        println!("Unlinking backup: {}", &backup);
        remove_file(backup)?;
    }

    Ok(())
}
