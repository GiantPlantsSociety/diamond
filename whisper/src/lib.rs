use crate::utils::{AsyncReadBytesExt, AsyncWriteBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::SeekFrom;
use std::path::Path;
use tokio::fs;
use tokio::io::{self, AsyncRead, AsyncSeek, AsyncWrite, AsyncWriteExt};
use tokio::prelude::*;

/*
# This module is an implementation of the Whisper database API
# Here is the basic layout of a whisper data file
#
# File = Header,Data
#   Header = Metadata,ArchiveInfo+
#       Metadata = aggregationType,maxRetention,xFilesFactor,archiveCount
#       ArchiveInfo = Offset,SecondsPerPoint,Points
#   Data = Archive+
#       Archive = Point+
#           Point = timestamp,value

try:
  if sys.version_info >= (3, 0):
    from os import posix_fadvise, POSIX_FADV_RANDOM
  else:
    from fadvise import posix_fadvise, POSIX_FADV_RANDOM
  CAN_FADVISE = True
except ImportError:
  CAN_FADVISE = False

LOCK = False
CACHE_HEADERS = False
AUTOFLUSH = False
FADVISE_RANDOM = False
# Buffering setting applied to all operations that do *not* require
# a full scan of the file in order to minimize cache thrashing.
BUFFERING = 0
__headerCache = {}
*/

pub mod aggregation;
pub mod archive_info;
pub mod builder;
pub mod diff;
pub mod error;
mod fallocate;
pub mod fill;
pub mod interval;
pub mod merge;
pub mod point;
pub mod resize;
pub mod retention;
pub mod utils;

use crate::aggregation::*;
use crate::archive_info::*;
use crate::interval::*;
use crate::point::*;

pub use crate::builder::WhisperBuilder;

pub const METADATA_SIZE: usize = 16;
pub const ARCHIVE_INFO_SIZE: usize = 12;
pub const POINT_SIZE: usize = 12;

#[derive(Debug, Clone)]
pub struct WhisperMetadata {
    pub aggregation_method: AggregationMethod,
    pub max_retention: u32,
    pub x_files_factor: f32,
    pub archives: Vec<ArchiveInfo>,
}

impl WhisperMetadata {
    pub async fn read<R: AsyncRead + AsyncSeek + Unpin + Send>(
        fh: &mut R,
    ) -> Result<Self, io::Error> {
        fh.seek(SeekFrom::Start(0)).await?;

        let aggregation_type = fh.read_u32().await?;
        let max_retention = fh.read_u32().await?;
        let x_files_factor = fh.aread_f32().await?;
        let archive_count = fh.read_u32().await?;

        let aggregation_method =
            AggregationMethod::from_type(aggregation_type).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Bad aggregation method {}", aggregation_type),
                )
            })?;

        if x_files_factor < 0.0 || x_files_factor > 1.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Bad x_files_factor {}", x_files_factor),
            ));
        }

        let mut archives = Vec::with_capacity(archive_count as usize);
        for _ in 0..archive_count {
            let archive_info = ArchiveInfo::read(fh).await?;
            archives.push(archive_info);
        }

        Ok(WhisperMetadata {
            aggregation_method,
            max_retention,
            x_files_factor,
            archives,
        })
    }

    fn header_size(&self) -> usize {
        METADATA_SIZE + ARCHIVE_INFO_SIZE * self.archives.len()
    }

    pub fn file_size(&self) -> usize {
        self.header_size()
            + self
                .archives
                .iter()
                .map(|archive| archive.points as usize * POINT_SIZE)
                .sum::<usize>()
    }

    async fn write_metadata<W: AsyncWrite + Unpin + Send>(
        &self,
        w: &mut W,
    ) -> Result<(), io::Error> {
        w.write_u32(self.aggregation_method.to_type()).await?;
        w.write_u32(self.max_retention).await?;
        w.awrite_f32(self.x_files_factor).await?;
        w.write_u32(self.archives.len() as u32).await?;
        Ok(())
    }

    async fn write<W: AsyncWrite + Unpin + Send>(&self, w: &mut W) -> Result<(), io::Error> {
        self.write_metadata(w).await?;
        for archive in &self.archives {
            archive.write(w).await?;
        }
        Ok(())
    }
}

pub struct WhisperFile {
    metadata: WhisperMetadata,
    file: fs::File,
}

impl WhisperFile {
    async fn create<P: AsRef<Path>>(
        header: &WhisperMetadata,
        path: P,
        sparse: bool,
    ) -> Result<Self, io::Error> {
        let mut metainfo_bytes = Vec::<u8>::new();
        header.write(&mut metainfo_bytes).await?;

        let mut fh = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .await?;

        // if LOCK {
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)
        // }

        fh.write_all(&metainfo_bytes).await?;
        if sparse {
            fh.seek(SeekFrom::Start(header.file_size() as u64 - 1))
                .await?;
            fh.write_all(&[0u8]).await?;
        } else {
            fallocate::fallocate(
                &mut fh,
                header.header_size(),
                header.file_size() - header.header_size(),
            )
            .await?;
        }

        fh.sync_all().await?;

        Ok(Self {
            metadata: header.clone(),
            file: fh,
        })
    }

    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .await?;
        let metadata = WhisperMetadata::read(&mut file).await?;
        Ok(Self { metadata, file })
    }

    pub fn info(&self) -> &WhisperMetadata {
        &self.metadata
    }

    pub async fn set_x_files_factor(&mut self, x_files_factor: f32) -> Result<(), io::Error> {
        if x_files_factor < 0.0 || x_files_factor > 1.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Bad x_files_factor {}", x_files_factor),
            ));
        }

        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        self.file.seek(SeekFrom::Start(0)).await?;
        self.metadata.x_files_factor = x_files_factor; // TODO: transactional update
        self.metadata.write_metadata(&mut self.file).await?;
        self.file.sync_data().await?;

        Ok(())
    }

    pub async fn set_aggregation_method(
        &mut self,
        aggregation_method: AggregationMethod,
    ) -> Result<(), io::Error> {
        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        self.file.seek(SeekFrom::Start(0)).await?;
        self.metadata.aggregation_method = aggregation_method; // TODO: transactional update
        self.metadata.write_metadata(&mut self.file).await?;
        self.file.sync_data().await?;

        Ok(())
    }

    pub async fn update(&mut self, point: &Point, now: u32) -> Result<(), io::Error> {
        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)
        file_update(&mut self.file, &self.metadata, point, now).await
    }

    pub async fn update_many(&mut self, points: &[Point], now: u32) -> Result<(), io::Error> {
        if points.is_empty() {
            return Ok(());
        }

        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        // if CAN_FADVISE and FADVISE_RANDOM:
        //     posix_fadvise(fh.fileno(), 0, 0, POSIX_FADV_RANDOM)

        let mut points_vec = points.to_vec();
        points_vec.sort_by_key(|p| std::u32::MAX - p.interval); // Order points by timestamp, newest first
        file_update_many(&mut self.file, &self.metadata, &points_vec, now).await
    }

    fn find_archive(&self, seconds_per_point: u32) -> Result<ArchiveInfo, io::Error> {
        self.metadata
            .archives
            .iter()
            .find(|archive| archive.seconds_per_point == seconds_per_point)
            .map(|a| a.to_owned())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Archive not found"))
    }

    pub fn suggest_archive(&self, interval: Interval, now: u32) -> Option<u32> {
        let meta = self.info();

        let adjusted = Interval::past(now, meta.max_retention)
            .intersection(interval)
            .ok()?;

        meta.archives
            .iter()
            .filter(|archive| Interval::past(now, archive.retention()).contains(adjusted))
            .map(|archive| archive.seconds_per_point)
            .next()
    }

    pub async fn fetch_points(
        &mut self,
        seconds_per_point: u32,
        interval: Interval,
        now: u32,
    ) -> Result<(Interval, Option<Vec<Point>>), io::Error> {
        let archive = self.find_archive(seconds_per_point)?;
        let available = Interval::past(now, self.metadata.max_retention);

        if !interval.intersects(available) {
            // Range is in the future or beyond retention
            return Ok((interval, None));
        }

        let interval = available
            .intersection(interval)
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

        let adjusted_interval = adjust_interval(interval, archive.seconds_per_point)
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

        let points = archive_fetch_interval(&mut self.file, &archive, adjusted_interval).await?;

        Ok((adjusted_interval, points))
    }

    pub async fn fetch_auto_points(
        &mut self,
        interval: Interval,
        now: u32,
    ) -> Result<ArchiveData, io::Error> {
        let seconds_per_point = self
            .suggest_archive(interval, now)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No data in selected timerange"))?;

        let (adjusted_interval, points) =
            self.fetch_points(seconds_per_point, interval, now).await?;
        let data = points_to_data(&points, adjusted_interval, seconds_per_point);
        Ok(data)
    }

    pub async fn fetch(
        &mut self,
        seconds_per_point: u32,
        interval: Interval,
        now: u32,
    ) -> Result<ArchiveData, io::Error> {
        let (adjusted_interval, points) =
            self.fetch_points(seconds_per_point, interval, now).await?;
        let data = points_to_data(&points, adjusted_interval, seconds_per_point);
        Ok(data)
    }

    pub async fn dump(&mut self, seconds_per_point: u32) -> Result<Vec<Point>, io::Error> {
        let archive = self.find_archive(seconds_per_point)?;
        read_archive(&mut self.file, &archive, 0, archive.points).await
    }
}

fn instant_offset(archive: &ArchiveInfo, base_interval: u32, instant: u32) -> u32 {
    #[inline]
    fn modulo(a: u32, b: u32) -> u32 {
        (a + b) % b
    }

    if base_interval == 0 {
        0
    } else {
        let instant_aligned = modulo(instant / archive.seconds_per_point, archive.points);
        let base_aligned = modulo(base_interval / archive.seconds_per_point, archive.points);
        modulo(
            archive.points + instant_aligned - base_aligned,
            archive.points,
        )
    }
}

async fn read_archive<R: AsyncRead + AsyncSeek + Unpin + Send>(
    fh: &mut R,
    archive: &ArchiveInfo,
    from_index: u32,
    until_index: u32,
) -> Result<Vec<Point>, io::Error> {
    let from_index = from_index % archive.points;
    let until_index = until_index % archive.points;

    let mut series =
        Vec::with_capacity(((archive.points + until_index - from_index) % archive.points) as usize);

    let point_size = 12;
    let from_offset = archive.offset + from_index * point_size;

    fh.seek(SeekFrom::Start(from_offset.into())).await?;
    if from_index < until_index {
        // If we don't wrap around the archive
        for _i in from_index..until_index {
            series.push(Point::read(fh).await?);
        }
    } else {
        // We do wrap around the archive, so we need two reads
        for _i in from_index..archive.points {
            series.push(Point::read(fh).await?);
        }
        fh.seek(SeekFrom::Start(archive.offset.into())).await?;
        for _i in 0..until_index {
            series.push(Point::read(fh).await?);
        }
    }

    Ok(series)
}

async fn write_archive_point<F: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send>(
    fh: &mut F,
    archive: &ArchiveInfo,
    point: &Point,
) -> Result<(), io::Error> {
    let base = archive.read_base(fh).await?;
    let index = instant_offset(archive, base.interval, point.interval);
    fh.seek(SeekFrom::Start(
        (archive.offset + index * POINT_SIZE as u32).into(),
    ))
    .await?;
    point.write(fh).await?;
    Ok(())
}

async fn write_archive<F: AsyncWrite + AsyncSeek + Unpin + Send>(
    fh: &mut F,
    archive: &ArchiveInfo,
    points: &[Point],
    base_interval: u32,
) -> Result<(), io::Error> {
    let point_size = 12;

    let first_interval = points[0].interval;

    let offset = instant_offset(archive, base_interval, first_interval);

    let available_tail_space = (archive.points - offset) as usize;

    if available_tail_space < points.len() {
        let (tail, head) = points.split_at(available_tail_space);

        fh.seek(SeekFrom::Start(
            (archive.offset + offset * point_size).into(),
        ))
        .await?;
        for point in tail {
            point.write(fh).await?;
        }
        fh.seek(SeekFrom::Start(archive.offset.into())).await?;
        for point in head {
            point.write(fh).await?;
        }
    } else {
        fh.seek(SeekFrom::Start(
            (archive.offset + offset * point_size).into(),
        ))
        .await?;
        for point in points {
            point.write(fh).await?;
        }
    }

    Ok(())
}

fn points_to_values(points: &[Point], start: u32, step: u32) -> Vec<Option<f64>> {
    let mut values = Vec::with_capacity(points.len());
    for (i, point) in points.iter().enumerate() {
        if point.interval == start + (i as u32) * step {
            values.push(Some(point.value));
        } else {
            values.push(None);
        }
    }
    values
}

async fn __propagate<F: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send>(
    fh: &mut F,
    header: &WhisperMetadata,
    timestamp: u32,
    higher: &ArchiveInfo,
    lower: &ArchiveInfo,
) -> Result<bool, io::Error> {
    let lower_interval_start = timestamp - (timestamp % lower.seconds_per_point);

    fh.seek(SeekFrom::Start(higher.offset.into())).await?;
    let higher_base = Point::read(fh).await?;

    let higher_first_index = instant_offset(higher, higher_base.interval, lower_interval_start);

    let higher_last_index = {
        let higher_points = lower.seconds_per_point / higher.seconds_per_point;
        (higher_first_index + higher_points) % higher.points
    };

    let series = read_archive(fh, higher, higher_first_index, higher_last_index).await?;

    // And finally we construct a list of values
    let neighbor_values = points_to_values(&series, lower_interval_start, higher.seconds_per_point);

    // Propagate aggregateValue to propagate from neighborValues if we have enough known points
    let known_values = neighbor_values.iter().filter(|v| v.is_some()).count();
    if known_values == 0 {
        return Ok(false);
    }

    let known_percent = known_values as f32 / neighbor_values.len() as f32;
    if known_percent >= header.x_files_factor {
        // We have enough data to propagate a value!
        let aggregate_value = header
            .aggregation_method
            .aggregate(&neighbor_values)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let my_point = Point {
            interval: lower_interval_start,
            value: aggregate_value,
        };

        write_archive_point(fh, lower, &my_point).await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

async fn file_update(
    fh: &mut fs::File,
    header: &WhisperMetadata,
    point: &Point,
    now: u32,
) -> Result<(), io::Error> {
    let timestamp = point.interval;

    if now >= timestamp + header.max_retention || now < timestamp {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Timestamp not covered by any archives in this database.",
        ));
    }

    // Find the highest-precision archive that covers timestamp
    let archive_index = header
        .archives
        .iter()
        .position(|a| timestamp + a.retention() >= now)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Timestamp not covered by any archives in this database.",
            )
        })?;

    let archive = &header.archives[archive_index];

    // First we update the highest-precision archive
    let interval = timestamp - (timestamp % archive.seconds_per_point);
    let adjusted_point = Point {
        interval,
        value: point.value,
    };

    write_archive_point(fh, archive, &adjusted_point).await?;

    // Now we propagate the update to lower-precision archives
    for pair in header.archives[archive_index..].windows(2) {
        let higher = &pair[0];
        let lower = &pair[1];
        if !__propagate(fh, &header, interval, higher, lower).await? {
            break;
        }
    }

    Ok(())
}

async fn file_update_many(
    fh: &mut fs::File,
    header: &WhisperMetadata,
    points: &[Point],
    now: u32,
) -> Result<(), io::Error> {
    let mut archive_index = 0;
    let mut current_points = vec![];

    for point in points {
        while point.interval + header.archives[archive_index].retention() < now {
            // We can't fit any more points in this archive
            if !current_points.is_empty() {
                // Commit all the points we've found that it can fit
                current_points.reverse(); // Put points in chronological order
                __archive_update_many(fh, &header, archive_index, &current_points).await?;
                current_points.clear();
            }
            archive_index += 1;
            if archive_index >= header.archives.len() {
                break;
            }
        }

        if archive_index >= header.archives.len() {
            break; // Drop remaining points that don't fit in the database
        }

        current_points.push(*point);
    }

    // Don't forget to commit after we've checked all the archives
    if archive_index < header.archives.len() && !current_points.is_empty() {
        current_points.reverse();
        __archive_update_many(fh, &header, archive_index, &current_points).await?;
    }

    Ok(())
}

/**
 * Create a packed string for each contiguous sequence of points
 * It's expected that points are sorted in chronological order
 */
fn pack_points(points: &[Point], step: u32) -> Vec<Vec<Point>> {
    let mut chunks: Vec<Vec<Point>> = Vec::new();
    let mut previous_interval: Option<u32> = None;
    let mut current_chunk: Vec<Point> = Vec::new();
    let len = points.len();
    for (i, point) in points.iter().enumerate() {
        // Take last point in run of points with duplicate intervals
        if i + 1 < len && points[i].interval == points[i + 1].interval {
            continue;
        }
        // (interval, value) = alignedPoints[i]
        if previous_interval.is_none() || Some(point.interval - step) == previous_interval {
            current_chunk.push(*point);
        } else {
            chunks.push(current_chunk);
            current_chunk = vec![*point];
        }
        previous_interval = Some(point.interval);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/**
 * It's expected that points are sorted in chronological order
 */
async fn __archive_update_many<F: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send>(
    fh: &mut F,
    header: &WhisperMetadata,
    archive_index: usize,
    points: &[Point],
) -> Result<(), io::Error> {
    let archive = &header.archives[archive_index];

    let aligned_points: Vec<Point> = points
        .iter()
        .map(|p| p.align(archive.seconds_per_point))
        .collect();

    let chunks: Vec<Vec<Point>> = pack_points(&aligned_points, archive.seconds_per_point);

    // Read base point and determine where our writes will start
    let base = archive.read_base(fh).await?;

    let base_interval = if base.interval == 0 {
        // This file's first update
        chunks[0][0].interval // Use our first string as the base, so we start at the start
    } else {
        base.interval
    };

    // Write all of our packed strings in locations determined by the baseInterval
    for chunk in chunks {
        write_archive(fh, archive, &chunk, base_interval).await?;
    }

    // Now we propagate the updates to lower-precision archives
    for pair in header.archives[archive_index..].windows(2) {
        let higher = &pair[0];
        let lower = &pair[1];

        let unique_lower_intervals: HashSet<u32> = aligned_points
            .iter()
            .map(|p| p.align(lower.seconds_per_point).interval)
            .collect();
        let mut propagate_further = false;
        for interval in unique_lower_intervals {
            if __propagate(fh, header, interval, higher, lower).await? {
                propagate_further = true;
            }
        }

        if !propagate_further {
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ArchiveData {
    #[serde(rename = "start")]
    pub from_interval: u32,
    #[serde(rename = "end")]
    pub until_interval: u32,
    pub step: u32,
    pub values: Vec<Option<f64>>,
}

impl ArchiveData {
    pub fn points(&self) -> Vec<Point> {
        (self.from_interval..self.until_interval)
            .step_by(self.step as usize)
            .zip(&self.values)
            .filter_map(|(interval, value)| value.map(|value| Point { interval, value }))
            .collect()
    }

    pub fn filter_out(&self, f: &dyn Fn(&Option<f64>) -> bool) -> ArchiveData {
        ArchiveData {
            values: self.values.clone().into_iter().filter(f).collect(),
            ..*self
        }
    }
}

fn adjust_instant(instant: u32, step: u32) -> u32 {
    instant - (instant % step)
}

fn adjust_instant_up(instant: u32, step: u32) -> u32 {
    (instant + step - 1) / step * step
}

fn adjust_interval(interval: Interval, step: u32) -> Result<Interval, String> {
    let from_interval = adjust_instant(interval.from(), step);
    let until_interval = adjust_instant_up(interval.until(), step);

    if from_interval == until_interval {
        // Zero-length time range: always include the next point
        Interval::new(from_interval, until_interval + step)
    } else {
        Interval::new(from_interval, until_interval)
    }
}

async fn archive_fetch_interval<R: AsyncRead + AsyncSeek + Unpin + Send>(
    fh: &mut R,
    archive: &ArchiveInfo,
    interval: Interval,
) -> Result<Option<Vec<Point>>, io::Error> {
    let base = archive.read_base(fh).await?;
    if base.interval == 0 {
        Ok(None)
    } else {
        let from_index = instant_offset(archive, base.interval, interval.from());
        let until_index = instant_offset(archive, base.interval, interval.until());
        let points = read_archive(fh, &archive, from_index, until_index).await?;
        Ok(Some(points))
    }
}

fn points_to_data(
    points: &Option<Vec<Point>>,
    interval: Interval,
    seconds_per_point: u32,
) -> ArchiveData {
    let values = match points {
        None => {
            let count = (interval.until() - interval.from()) / seconds_per_point;
            vec![None; count as usize]
        }
        Some(points) => points_to_values(&points, interval.from(), seconds_per_point),
    };

    ArchiveData {
        from_interval: interval.from(),
        until_interval: interval.until(),
        step: seconds_per_point,
        values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_offset() {
        let archive = ArchiveInfo {
            offset: 100_000,
            seconds_per_point: 1,
            points: 60,
        };

        assert_eq!(instant_offset(&archive, 0, 0), 0);
        assert_eq!(instant_offset(&archive, 0, 1), 0);

        assert_eq!(instant_offset(&archive, 10, 10), 0);
        assert_eq!(instant_offset(&archive, 10, 11), 1);
        assert_eq!(instant_offset(&archive, 10, 12), 2);
        assert_eq!(instant_offset(&archive, 10, 9), 59);
        assert_eq!(instant_offset(&archive, 10, 8), 58);

        assert_eq!(instant_offset(&archive, 10, 50), 40);
        assert_eq!(instant_offset(&archive, 10, 69), 59);
        assert_eq!(instant_offset(&archive, 10, 70), 0);

        assert_eq!(instant_offset(&archive, 10, 120 + 50), 40);
        assert_eq!(instant_offset(&archive, 10, 120 + 69), 59);
        assert_eq!(instant_offset(&archive, 10, 120 + 70), 0);
    }

    #[test]
    fn test_adjust_interval() {
        assert_eq!(
            adjust_interval(Interval::new(1, 1).unwrap(), 10).unwrap(),
            Interval::new(0, 10).unwrap()
        );
        assert_eq!(
            adjust_interval(Interval::new(1, 5).unwrap(), 10).unwrap(),
            Interval::new(0, 10).unwrap()
        );
        assert_eq!(
            adjust_interval(Interval::new(0, 10).unwrap(), 10).unwrap(),
            Interval::new(0, 10).unwrap()
        );
        assert_eq!(
            adjust_interval(Interval::new(0, 11).unwrap(), 10).unwrap(),
            Interval::new(0, 20).unwrap()
        );
    }
}
