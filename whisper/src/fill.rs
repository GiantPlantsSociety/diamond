use super::*;
use interval::Interval;
use std::io;
use std::path::Path;

fn fill(src: &Path, dst: &Path, tstart: u32, mut tstop: u32, now: u32) -> Result<(), io::Error> {
    //     # find oldest point in time, stored by both files
    //     src_time = int(time.time()) - srcHeader['maxRetention']
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
        // let data_src = file_dst
        //    .fetch(archive.seconds_per_point, interval, now)?
        //    .unwrap();

        // let mut start = data_src.from_interval;
        // let end = data_src.until_interval;
        // let archive_step = data_src.step;

        let (_adjusted_interval, points) =
            file_src.fetch_points(archive.seconds_per_point, interval, now)?;

        if let Some(ref points) = points {
            file_dst.update_many(points, now)?;
        }

        tstop = from_time;

        // can stop when there's nothing to fetch any more
        if tstart == tstop {
            return Ok(());
        }
    }
    Ok(())
}

pub fn fill_archives(
    src: &Path,
    dst: &Path,
    mut start_from: u32,
    now: u32,
) -> Result<(), io::Error> {
    // let mut file_src = WhisperFile::open(src)?;
    let mut file_dst = WhisperFile::open(dst)?;

    let mut archives = file_dst.info().archives.clone();
    archives.sort_by_key(|a| a.retention());

    for archive in &archives {
        let from_time = now - archive.retention();
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

        let mut gapstart = 0;

        for v in data_dst.values {
            if !(v.is_some()) && !(gapstart > 0) {
                gapstart = start;
            } else if v.is_some() && gapstart > 0 {
                if (start - gapstart) > archive.seconds_per_point {
                    fill(src, dst, gapstart - step, start, now)?;
                }
                gapstart = 0;
            } else if (gapstart > 0) && (start == (end - step)) {
                fill(src, dst, gapstart - step, start, now)?;
            }
            start += step;
        }

        start_from = from_time
    }
    Ok(())
}
