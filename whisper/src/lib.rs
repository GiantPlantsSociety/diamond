extern crate failure;
extern crate regex;
extern crate lazy_static;
extern crate byteorder;
extern crate libc;
extern crate serde_derive;

use std::io::{self, Read, Write, Seek};
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use serde_derive::*;

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


pub mod error;
pub mod interval;
pub mod aggregation;
pub mod retention;
pub mod point;
pub mod archive_info;
pub mod builder;
mod fallocate;
pub mod diff;
pub mod merge;
pub mod fill;

use crate::interval::*;
use crate::aggregation::*;
use crate::point::*;
use crate::archive_info::*;

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
    pub fn read<R: Read + Seek>(fh: &mut R) -> Result<Self, io::Error> {
        fh.seek(io::SeekFrom::Start(0))?;

        let aggregation_type = fh.read_u32::<BigEndian>()?;
        let max_retention = fh.read_u32::<BigEndian>()?;
        let x_files_factor = fh.read_f32::<BigEndian>()?;
        let archive_count = fh.read_u32::<BigEndian>()?;

        let aggregation_method = AggregationMethod::from_type(aggregation_type)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, format!("Bad aggregation method {}", aggregation_type)))?;

        if x_files_factor < 0.0 || x_files_factor > 1.0 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Bad x_files_factor {}", x_files_factor)));
        }

        let mut archives = Vec::with_capacity(archive_count as usize);
        for _ in 0..archive_count {
            let archive_info = ArchiveInfo::read(fh)?;
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
        self.header_size() + self.archives.iter().map(|archive| archive.points as usize * POINT_SIZE).sum::<usize>()
    }

    fn write_metadata<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_u32::<BigEndian>(self.aggregation_method.to_type())?;
        w.write_u32::<BigEndian>(self.max_retention)?;
        w.write_f32::<BigEndian>(self.x_files_factor)?;
        w.write_u32::<BigEndian>(self.archives.len() as u32)?;
        Ok(())
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
        self.write_metadata(w)?;
        for archive in &self.archives {
            archive.write(w)?;
        }
        Ok(())
    }
}

pub struct WhisperFile {
    metadata: WhisperMetadata,
    file: fs::File,
}

impl WhisperFile {
    fn create(header: &WhisperMetadata, path: &Path, sparse: bool) -> Result<Self, io::Error> {
        let mut metainfo_bytes = Vec::<u8>::new();
        header.write(&mut metainfo_bytes)?;

        let mut fh = fs::OpenOptions::new().read(true).write(true).create_new(true).open(path)?;

        // if LOCK {
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)
        // }

        fh.write_all(&metainfo_bytes)?;
        if sparse {
            fh.seek(io::SeekFrom::Start(header.file_size() as u64 - 1))?;
            fh.write_all(&[0u8])?;
        } else {
            fallocate::fallocate(&mut fh, header.header_size(), header.file_size() - header.header_size())?;
        }

        fh.sync_all()?;

        Ok(Self {
            metadata: header.clone(),
            file: fh,
        })
    }

    pub fn open(path: &Path) -> Result<Self, io::Error> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let metadata = WhisperMetadata::read(&mut file)?;
        Ok(Self {
            metadata,
            file,
        })
    }

    pub fn info(&self) -> &WhisperMetadata {
        &self.metadata
    }

    pub fn set_x_files_factor(&mut self, x_files_factor: f32) -> Result<(), io::Error> {
        if x_files_factor < 0.0 || x_files_factor > 1.0 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Bad x_files_factor {}", x_files_factor)));
        }

        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        self.file.seek(io::SeekFrom::Start(0))?;
        self.metadata.x_files_factor = x_files_factor; // TODO: transactional update
        self.metadata.write_metadata(&mut self.file)?;
        self.file.sync_data()?;

        Ok(())
    }

    pub fn set_aggregation_method(&mut self, aggregation_method: AggregationMethod) -> Result<(), io::Error> {
        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        self.file.seek(io::SeekFrom::Start(0))?;
        self.metadata.aggregation_method = aggregation_method; // TODO: transactional update
        self.metadata.write_metadata(&mut self.file)?;
        self.file.sync_data()?;

        Ok(())
    }

    pub fn update(&mut self, point: &Point,  now: u32) -> Result<(), io::Error> {
        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)
        file_update(&mut self.file, &self.metadata, point, now)
    }

    pub fn update_many(&mut self, points: &[Point], now: u32) -> Result<(), io::Error> {
        if points.is_empty() {
            return Ok(());
        }

        // if LOCK:
        //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

        // if CAN_FADVISE and FADVISE_RANDOM:
        //     posix_fadvise(fh.fileno(), 0, 0, POSIX_FADV_RANDOM)

        let mut points_vec = points.to_vec();
        points_vec.sort_by_key(|p| std::u32::MAX - p.interval); // Order points by timestamp, newest first
        file_update_many(&mut self.file, &self.metadata, &points_vec, now)
    }

    fn find_archive(&self, seconds_per_point: u32) -> Result<ArchiveInfo, io::Error> {
        self.metadata.archives.iter()
            .find(|archive| archive.seconds_per_point == seconds_per_point)
            .map(|a| a.to_owned())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Archive not found"))
    }

    pub fn fetch_points(&mut self, seconds_per_point: u32, interval: Interval, now: u32) -> Result<(Interval, Option<Vec<Point>>), io::Error> {
        let archive = self.find_archive(seconds_per_point)?;
        let available = Interval::past(now, self.metadata.max_retention);

        if !interval.intersects(available) {
            // Range is in the future or beyond retention
            return Ok((interval, None));
        }

        let interval = available.intersection(interval)
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

        let adjusted_interval = adjust_interval(interval, archive.seconds_per_point)
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

        let points = archive_fetch_interval(&mut self.file, &archive, adjusted_interval)?;

        Ok((adjusted_interval, points))
    }

    pub fn fetch(&mut self, seconds_per_point: u32, interval: Interval, now: u32) -> Result<Option<ArchiveData>, io::Error> {
        let (adjusted_interval, points) = self.fetch_points(seconds_per_point, interval, now)?;
        let data = points_to_data(&points, adjusted_interval, seconds_per_point);
        Ok(Some(data))
    }

    pub fn dump(&mut self, seconds_per_point: u32) -> Result<Vec<Point>, io::Error> {
        let archive = self.find_archive(seconds_per_point)?;
        read_archive(&mut self.file, &archive, 0, archive.points)
    }
}

pub fn suggest_archive(file: &WhisperFile, interval: Interval, now: u32) -> Option<u32> {
    let meta = file.info();

    let adjusted = Interval::past(now, meta.max_retention)
        .intersection(interval).ok()?;

    meta.archives.iter()
        .filter(|archive| Interval::past(now, archive.retention()).contains(adjusted))
        .map(|archive| archive.seconds_per_point)
        .next()
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
        modulo(archive.points + instant_aligned - base_aligned, archive.points)
    }
}

fn read_archive<R: Read + Seek>(fh: &mut R, archive: &ArchiveInfo, from_index: u32, until_index: u32) -> Result<Vec<Point>, io::Error> {
    let from_index = from_index % archive.points;
    let until_index = until_index % archive.points;

    let mut series = Vec::with_capacity(((archive.points + until_index - from_index) % archive.points) as usize);

    let point_size = 12;
    let from_offset = archive.offset + from_index * point_size;

    fh.seek(io::SeekFrom::Start(from_offset.into()))?;
    if from_index < until_index {
        // If we don't wrap around the archive
        for _i in from_index..until_index {
            series.push(Point::read(fh)?);
        }
    } else {
        // We do wrap around the archive, so we need two reads
        for _i in from_index..archive.points {
            series.push(Point::read(fh)?);
        }
        fh.seek(io::SeekFrom::Start(archive.offset.into()))?;
        for _i in 0..until_index {
            series.push(Point::read(fh)?);
        }
    }

    Ok(series)
}

fn write_archive_point<F: Read + Write + Seek>(fh: &mut F, archive: &ArchiveInfo, point: &Point) -> Result<(), io::Error> {
    let base = archive.read_base(fh)?;
    let index = instant_offset(archive, base.interval, point.interval);
    fh.seek(io::SeekFrom::Start((archive.offset + index * POINT_SIZE as u32).into()))?;
    point.write(fh)?;
    Ok(())
}

fn write_archive<F: Write + Seek>(fh: &mut F, archive: &ArchiveInfo, points: &[Point], base_interval: u32) -> Result<(), io::Error> {
    let point_size = 12;

    let first_interval = points[0].interval;

    let offset = instant_offset(archive, base_interval, first_interval);

    let available_tail_space = (archive.points - offset) as usize;

    if available_tail_space < points.len() {
        let (tail, head) = points.split_at(available_tail_space);

        fh.seek(io::SeekFrom::Start((archive.offset + offset * point_size).into()))?;
        for point in tail {
            point.write(fh)?;
        }
        fh.seek(io::SeekFrom::Start(archive.offset.into()))?;
        for point in head {
            point.write(fh)?;
        }
    } else {
        fh.seek(io::SeekFrom::Start((archive.offset + offset * point_size).into()))?;
        for point in points {
            point.write(fh)?;
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

fn __propagate<F: Read + Write + Seek>(fh: &mut F, header: &WhisperMetadata, timestamp: u32, higher: &ArchiveInfo, lower: &ArchiveInfo) -> Result<bool, io::Error> {
    let lower_interval_start = timestamp - (timestamp % lower.seconds_per_point);

    fh.seek(io::SeekFrom::Start(higher.offset.into()))?;
    let higher_base = Point::read(fh)?;

    let higher_first_index = instant_offset(higher, higher_base.interval, lower_interval_start);

    let higher_last_index = {
        let higher_points = lower.seconds_per_point / higher.seconds_per_point;
        (higher_first_index + higher_points) % higher.points
    };

    let series = read_archive(fh, higher, higher_first_index, higher_last_index)?;

    // And finally we construct a list of values
    let neighbor_values = points_to_values(&series, lower_interval_start, higher.seconds_per_point);

    // Propagate aggregateValue to propagate from neighborValues if we have enough known points
    let known_values = neighbor_values.iter().filter(|v| v.is_some()).count();
    if known_values == 0 {
        return Ok(false);
    }

    let known_percent = known_values as f32 / neighbor_values.len() as f32;
    if known_percent >= header.x_files_factor {  // We have enough data to propagate a value!
        let aggregate_value = header.aggregation_method.aggregate(&neighbor_values)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let my_point = Point { interval: lower_interval_start, value: aggregate_value };

        write_archive_point(fh, lower, &my_point)?;

        Ok(true)
    } else {
        Ok(false)
    }
}

fn file_update(fh: &mut fs::File, header: &WhisperMetadata, point: &Point, now: u32) -> Result<(), io::Error> {
    let timestamp = point.interval;

    if now >= timestamp + header.max_retention || now < timestamp {
        return Err(io::Error::new(io::ErrorKind::Other, "Timestamp not covered by any archives in this database."));
    }

    // Find the highest-precision archive that covers timestamp
    let archive_index = header.archives.iter()
        .position(|a| timestamp + a.retention() >= now)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Timestamp not covered by any archives in this database."))?;

    let archive = &header.archives[archive_index];

    // First we update the highest-precision archive
    let interval = timestamp - (timestamp % archive.seconds_per_point);
    let adjusted_point = Point { interval, value: point.value };

    write_archive_point(fh, archive, &adjusted_point)?;

    // Now we propagate the update to lower-precision archives
    for pair in header.archives[archive_index..].windows(2) {
        let higher = &pair[0];
        let lower = &pair[1];
        if !__propagate(fh, &header, interval, higher, lower)? {
            break;
        }
    }

    Ok(())
}

fn file_update_many(fh: &mut fs::File, header: &WhisperMetadata, points: &[Point], now: u32) -> Result<(), io::Error> {
    let mut archive_index = 0;
    let mut current_points = vec![];

    for point in points {
        while point.interval + header.archives[archive_index].retention() < now {  // We can't fit any more points in this archive
            if !current_points.is_empty() { // Commit all the points we've found that it can fit
                current_points.reverse();  // Put points in chronological order
                __archive_update_many(fh, &header, archive_index, &current_points)?;
                current_points.clear();
            }
            archive_index += 1;
            if archive_index >= header.archives.len() {
                break;
            }
        }

        if archive_index >= header.archives.len() {
            break;  // Drop remaining points that don't fit in the database
        }

        current_points.push(*point);
    }

    // Don't forget to commit after we've checked all the archives
    if archive_index < header.archives.len() && !current_points.is_empty() {
        current_points.reverse();
        __archive_update_many(fh, &header, archive_index, &current_points)?;
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
            current_chunk = vec![ *point ];
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
fn __archive_update_many<F: Read + Write + Seek>(fh: &mut F, header: &WhisperMetadata, archive_index: usize, points: &[Point]) -> Result<(), io::Error> {
    let archive = &header.archives[archive_index];

    let aligned_points: Vec<Point> = points.iter().map(|p| p.align(archive.seconds_per_point)).collect();

    let chunks: Vec<Vec<Point>> = pack_points(&aligned_points, archive.seconds_per_point);

    // Read base point and determine where our writes will start
    let base = archive.read_base(fh)?;

    let base_interval = if base.interval == 0 {
        // This file's first update
        chunks[0][0].interval  // Use our first string as the base, so we start at the start
    } else {
        base.interval
    };

    // Write all of our packed strings in locations determined by the baseInterval
    for chunk in chunks {
        write_archive(fh, archive, &chunk, base_interval)?;
    }

    // Now we propagate the updates to lower-precision archives
    for pair in header.archives[archive_index..].windows(2) {
        let higher = &pair[0];
        let lower = &pair[1];

        let unique_lower_intervals: HashSet<u32> = aligned_points.iter().map(|p| p.align(lower.seconds_per_point).interval).collect();
        let mut propagate_further = false;
        for interval in unique_lower_intervals {
            if __propagate(fh, header, interval, higher, lower)? {
                propagate_further = true;
            }
        }

        if !propagate_further {
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
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
        (self.from_interval..self.until_interval).step_by(self.step as usize)
            .zip(&self.values)
            .filter_map(|(interval, value)| value.map(|value| Point { interval, value }))
            .collect()
    }

    pub fn filter_out(&self, f: &Fn(&Option<f64>) -> bool) -> ArchiveData {
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

fn archive_fetch_interval<R: Read + Seek>(fh: &mut R, archive: &ArchiveInfo, interval: Interval) -> Result<Option<Vec<Point>>, io::Error> {
    let base = archive.read_base(fh)?;
    if base.interval == 0 {
        Ok(None)
    } else {
        let from_index = instant_offset(archive, base.interval, interval.from());
        let until_index = instant_offset(archive, base.interval, interval.until());
        let points = read_archive(fh, &archive, from_index, until_index)?;
        Ok(Some(points))
    }
}

fn points_to_data(points: &Option<Vec<Point>>, interval: Interval, seconds_per_point: u32) -> ArchiveData {
    let values = match points {
        None => {
            let count = (interval.until() - interval.from()) / seconds_per_point;
            vec![None; count as usize]
        },
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
        let archive = ArchiveInfo { offset: 100_000, seconds_per_point: 1, points: 60 };

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
}
