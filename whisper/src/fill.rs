use super::*;
use interval::Interval;
use std::io;
use std::path::Path;

fn fill_interval(
    src: &Path,
    dst: &Path,
    tstart: u32,
    tsuntil: u32,
    now: u32,
) -> Result<(), io::Error> {
    let mut tstop = tsuntil;

    let mut file_src = WhisperFile::open(src)?;
    let mut file_dst = WhisperFile::open(dst)?;

    let mut archives = file_src.info().archives.clone();
    archives.sort_by_key(|a| a.retention());

    // find oldest point in time, stored by both files
    let src_time = now - file_src.info().max_retention;

    if (tstart < src_time) && (tstop < src_time) {
        return Ok(());
    }

    // we want to retain as much precision as we can, hence we do backwards
    // walk in time
    // skip forward at max 'step' points at a time

    for archive in &archives {
        let rtime = now - archive.retention();
        if tstop <= rtime {
            continue;
        }

        let until_time = tstop;
        let from_time = if rtime > tstart { rtime } else { tstart };

        let interval = Interval::new(from_time, until_time).unwrap();

        let (_adjusted_interval, points) =
            file_src.fetch_points(archive.seconds_per_point, interval, now)?;

        if let Some(ref points) = points {
            // filter zero points
            let mut points_to_write: Vec<Point> = points
                .clone()
                .into_iter()
                .filter(|point| point.interval != 0)
                .collect();

            // order points by timestamp, newest first
            points_to_write.sort_by_key(|point| point.interval);
            points_to_write.reverse();

            file_dst.update_many(&points_to_write, now)?;
        }

        tstop = from_time;

        // can stop when there's nothing to fetch any more
        if tstart == tstop {
            return Ok(());
        }
    }
    Ok(())
}

pub fn fill(src: &Path, dst: &Path, from: u32, now: u32) -> Result<(), io::Error> {
    let mut start_from = from;
    let mut file_dst = WhisperFile::open(dst)?;

    let mut archives = file_dst.info().archives.clone();
    archives.sort_by_key(|a| a.retention());

    for archive in &archives {
        let mut from_time = now - archive.retention();

        if from_time >= start_from {
            continue;
        }

        let interval = Interval::new(from_time, start_from).unwrap();
        let data_dst = file_dst
            .fetch(archive.seconds_per_point, interval, now)?
            .unwrap();

        let mut start = data_dst.from_interval;
        let end = data_dst.until_interval;
        let step = data_dst.step;

        let mut gapstart: Option<u32> = None;

        for v in data_dst.values {
            if !(v.is_some()) && !(gapstart.is_some()) {
                gapstart = Some(start);
            } else if v.is_some() && gapstart.is_some() {
                let gapstart_unwrap = gapstart.unwrap();
                if (start - gapstart_unwrap) > archive.seconds_per_point {
                    fill_interval(src, dst, gapstart_unwrap, start, now)?;
                }
                gapstart = None;
            } else if (gapstart.is_some()) && (start == (end - step)) {
                fill_interval(src, dst, gapstart.unwrap(), start, now)?;
            }
            start += step;
        }

        start_from = from_time
    }
    Ok(())
}
