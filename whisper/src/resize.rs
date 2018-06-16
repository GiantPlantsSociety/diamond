use crate::aggregation::AggregationMethod;
use crate::builder::WhisperBuilder;
use crate::interval::Interval;

use crate::point::Point;
use crate::retention::Retention;
use crate::WhisperFile;

use failure::{err_msg, Error, format_err};
use std::fs::{remove_file, rename};
use std::path::PathBuf;
use std::process::exit;

fn migrate_points(
    path_src: &PathBuf,
    path_dst: &PathBuf,
    aggregate: bool,
    now: u32,
) -> Result<(), Error> {
    if !path_src.is_file() {
        return Err(format_err!(
            "[ERROR] File {} does not exist!\n",
            path_src.display()
        ));
    }

    let mut file_src = WhisperFile::open(&path_src)?;
    let meta = file_src.info().clone();

    let mut file_dst = WhisperFile::open(&path_dst)?;

    match aggregate {
        true => {
            // This is where data will be interpolated (best effort)
            println!("Migrating data with aggregation...");
            let mut until = now;

            for archive in &meta.archives {
                let interval = Interval::new(0, until).map_err(err_msg)?;

                let (adjusted_interval, data) =
                    file_src.fetch_points(archive.seconds_per_point, interval, now)?;

                if let Some(ref data) = data {
                    let mut points_to_write: Vec<Point> = data.clone()
                        .into_iter()
                        .filter(|point| point != &Point::default())
                        .collect();

                    points_to_write.sort_by_key(|point| point.interval);

                    println!(
                        "({},{},{})",
                        &adjusted_interval.from(),
                        &adjusted_interval.until(),
                        &archive.seconds_per_point
                    );
                    // timepoints_to_update = range(fromTime, untilTime, step)
                    // print("timepoints_to_update: %s" % timepoints_to_update)
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
        }

        false => {
            println!("Migrating data without aggregation...");
            let interval = Interval::new(0, now).map_err(err_msg)?;

            let mut archives = meta.archives;
            archives.sort_by_key(|archive| archive.retention());

            for archive in &archives {
                let (_adjusted_interval, data) =
                    file_src.fetch_points(archive.seconds_per_point, interval, now)?;

                if let Some(ref data) = data {
                    let mut points_to_write: Vec<Point> = data.clone()
                        .into_iter()
                        .filter(|point| point != &Point::default())
                        .collect();

                    points_to_write.sort_by_key(|point| point.interval);
                    file_dst.update_many(&points_to_write, now)?;
                }
            }
        }
    };

    return Ok(());
}

pub fn resize(
    path_src: &PathBuf,
    path_new: Option<PathBuf>,
    retentions: Vec<Retention>,
    x_files_factor: f32,
    aggregation_method: AggregationMethod,
    aggregate: bool,
    nobackup: bool,
    now: u32,
) -> Result<(), Error> {
    let path_dst = match path_new.clone() {
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
        Some(new) => new,
    };

    println!("Retrieving all data from the archives");
    println!("Creating new whisper database: {}", path_dst.display());

    WhisperBuilder::default()
        .add_retentions(&retentions)
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
    rename(&path_src, &backup)?;

    println!("Renaming new database to: {}", path_src.display());
    rename(&path_dst, &path_src)
        .map_err(|e| {
            eprintln!("{}", e);
            println!("Operation failed, restoring backup");
            rename(&backup, &path_src).unwrap();
            exit(1);
        })
        .unwrap();

    if nobackup {
        println!("Unlinking backup: {}", &backup);
        remove_file(backup)?;
    }

    Ok(())
}
