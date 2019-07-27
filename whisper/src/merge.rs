use super::*;
use crate::interval::Interval;
use std::io;
use std::path::Path;

/**
 * Merges the data from one whisper file into another. Each file must have
 * the same archive configuration. time_from and time_to can optionally be
 * specified for the merge.
 */
pub fn merge(
    path_src: &Path,
    path_dst: &Path,
    time_from: u32,
    time_to: u32,
    now: u32,
) -> Result<(), io::Error> {
    // if now is None:
    //     now = int(time.time())

    // if (time_to is not None):
    //     untilTime = time_to
    // else:
    //     untilTime = now

    // if (time_from is not None):
    //     fromTime = time_from
    // else:
    //     fromTime = 0

    let mut file_src = WhisperFile::open(path_src)?;
    let mut file_dst = WhisperFile::open(path_dst)?;

    if file_src.info().archives != file_dst.info().archives {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Archive configurations are unalike. Resize the input before merging",
        ));
    }

    // Sanity check: do not mix the from/to values.
    if time_to < time_from {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "time_to must be >= time_from",
        ));
    }

    let mut archives = file_src.info().archives.clone();
    archives.sort_by_key(|archive| archive.retention());

    for archive in &archives {
        // if time_to is too old, skip this archive
        if time_to < now - archive.retention() {
            continue;
        }

        let from = u32::max(time_from, now - archive.retention());
        let interval = Interval::new(from, time_to).unwrap();

        let (_adjusted_interval, points) =
            file_src.fetch_points(archive.seconds_per_point, interval, now)?;

        if let Some(ref points) = points {
            file_dst.update_many(points, now)?;
        }
    }
    Ok(())
}
