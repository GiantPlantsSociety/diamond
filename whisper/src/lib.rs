#[macro_use]
extern crate failure;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate byteorder;
extern crate libc;
extern crate num;

use std::io::{self, Read, Write, Seek};
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::range_step;

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
mod fallocate;

use error::{InvalidConfiguration};
use interval::*;
use aggregation::*;
use retention::*;
use point::*;
use archive_info::*;

const METADATA_SIZE: usize = 16;
const ARCHIVE_INFO_SIZE: usize = 12;
pub const POINT_SIZE: usize = 12;

#[derive(Debug)]
pub struct WhisperMetadata {
    pub aggregation_method: AggregationMethod,
    pub max_retention: u32,
    pub x_files_factor: f32,
    pub archives: Vec<ArchiveInfo>,
}

impl WhisperMetadata {
    pub fn create(retentions: &[Retention], x_files_factor: impl Into<Option<f32>>, aggregation_method: impl Into<Option<AggregationMethod>>) -> Result<Self, InvalidConfiguration> {
        let x = match x_files_factor.into() {
            None => 0.5,
            Some(value) if value >= 0.0 && value < 1.0 => value,
            Some(value) => return Err(InvalidConfiguration::InvalidXFilesFactor(value)),
        };

        if retentions.is_empty() {
            return Err(InvalidConfiguration::NoArchives);
        }

        let mut archives: Vec<_> = retentions.iter().map(|retention| ArchiveInfo {
            offset: 0,
            seconds_per_point: retention.seconds_per_point,
            points: retention.points,
        }).collect();

        archives.sort_by_key(|a| a.seconds_per_point);

        validate_archive_list(&archives)?;

        // update offsets
        let mut offset = METADATA_SIZE + ARCHIVE_INFO_SIZE * archives.len();
        for archive in &mut archives {
            archive.offset = offset as u32;
            offset += archive.points as usize * POINT_SIZE;
        }

        let max_retention = archives.iter().map(|archive| archive.retention()).max().unwrap();

        let header = WhisperMetadata {
            aggregation_method: aggregation_method.into().unwrap_or(AggregationMethod::Average),
            max_retention,
            x_files_factor: x,
            archives,
        };

        Ok(header)
    }

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

    pub fn write_metadata<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_u32::<BigEndian>(self.aggregation_method.to_type())?;
        w.write_u32::<BigEndian>(self.max_retention)?;
        w.write_f32::<BigEndian>(self.x_files_factor)?;
        w.write_u32::<BigEndian>(self.archives.len() as u32)?;
        Ok(())
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
        self.write_metadata(w)?;
        for archive in &self.archives {
            archive.write(w)?;
        }
        Ok(())
    }
}

/**
 * Validates an archiveList.
 *
 * An ArchiveList must:
 * 1. No archive may be a duplicate of another.
 * 2. Higher precision archives' precision must evenly divide all lower precision archives' precision.
 * 3. Lower precision archives must cover larger time intervals than higher precision archives.
 * 4. Each archive must have at least enough points to consolidate to the next archive
 */
fn validate_archive_list(archives: &[ArchiveInfo]) -> Result<(), InvalidConfiguration> {
    for (i, pair) in archives.windows(2).enumerate() {
        let archive = &pair[0];
        let next_archive = &pair[1];

        if archive.seconds_per_point >= next_archive.seconds_per_point {
            return Err(InvalidConfiguration::SamePrecision(i, *archive, *next_archive));
        }

        if next_archive.seconds_per_point % archive.seconds_per_point != 0 {
            return Err(InvalidConfiguration::UndividablePrecision(i, *archive, *next_archive));
        }

        let retention = archive.retention();
        let next_retention = next_archive.retention();

        if next_retention <= retention {
            return Err(InvalidConfiguration::BadRetention(i, retention, next_retention));
        }

        let points_per_consolidation = next_archive.seconds_per_point / archive.seconds_per_point;
        if archive.points < points_per_consolidation {
            return Err(InvalidConfiguration::NotEnoughPoints(i + 1, points_per_consolidation, archive.points));
        }
    }

    Ok(())
}

fn __read_header<R: Read + Seek>(fh: &mut R) -> Result<WhisperMetadata, io::Error> {
    // if CACHE_HEADERS {
    //     info = __headerCache.get(fh.name)
    //     if info {
    //         return info
    //     }
    // }

    let info = WhisperMetadata::read(fh)?;

    // if CACHE_HEADERS {
    //     __headerCache[fh.name] = info
    // }

    Ok(info)
}

/**
 * Sets the xFilesFactor for file in path
 *
 * path is a string pointing to a whisper file
 * xFilesFactor is a float between 0 and 1
 *
 * returns the old xFilesFactor
 */
pub fn set_x_files_factor(path: &Path, x_files_factor: f32) -> Result<f32, io::Error> {
    let info = __set_aggregation(path, None, Some(x_files_factor))?;
    Ok(info.x_files_factor)
}

/**
 * Sets the aggregationMethod for file in path
 *
 * path is a string pointing to the whisper file
 * aggregationMethod specifies the method to use when propagating data
 * xFilesFactor specifies the fraction of data points in a propagation interval
 * that must have known values for a propagation to occur. If None, the
 * existing xFilesFactor in path will not be changed
 *
 * returns the old aggregationMethod
 */
pub fn set_aggregation_method(path: &Path, aggregation_method: AggregationMethod, x_files_factor: Option<f32>) -> Result<AggregationMethod, io::Error> {
    let info = __set_aggregation(path, Some(aggregation_method), x_files_factor)?;
    Ok(info.aggregation_method)
}

/**
 * Set aggregationMethod and or xFilesFactor for file in path
 */
fn __set_aggregation(path: &Path, aggregation_method: Option<AggregationMethod>, x_files_factor: Option<f32>) -> Result<WhisperMetadata, io::Error> {
    let mut fh = fs::OpenOptions::new().read(true).write(true).open(path)?;

    // if LOCK:
    //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

    let mut info = __read_header(&mut fh)?;

    if let Some(aggregation_method) = aggregation_method {
        info.aggregation_method = aggregation_method;
    }
    if let Some(x_files_factor) = x_files_factor {
        // if x_files_factor < 0.0 || x_files_factor > 1.0 {
        //     return Err(Error::InvalidXFilesFactor(x_files_factor));
        // }
        info.x_files_factor = x_files_factor;
    }

    fh.seek(io::SeekFrom::Start(0))?;
    info.write_metadata(&mut fh)?;

    // if CACHE_HEADERS and fh.name in __headerCache:
    //     del __headerCache[fh.name]

    Ok(info)
}

/**
 * create(path,archiveList,xFilesFactor=0.5,aggregationMethod='average')
 *
 * path               is a string
 * archiveList        is a list of archives, each of which is of the form (secondsPerPoint, numberOfPoints)
 * xFilesFactor       specifies the fraction of data points in a propagation interval that must have known values for a propagation to occur
 * aggregationMethod  specifies the function to use when propagating data
 */
pub fn create(header: &WhisperMetadata, path: &Path, sparse: bool) -> Result<(), io::Error> {
    let mut metainfo_bytes = Vec::<u8>::new();
    header.write(&mut metainfo_bytes)?;

    let mut fh = fs::OpenOptions::new().write(true).create_new(true).open(path)?;

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

    Ok(())
}

pub fn read_archive_all(path: &Path, archive: &ArchiveInfo) -> Result<Vec<Point>, io::Error> {
    let mut fh = fs::OpenOptions::new().read(true).open(path)?;
    read_archive(&mut fh, &archive, 0, archive.points)
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
    let point_size = 12;

    let base = archive.read_base(fh)?;

    let index = if base.interval == 0 {
        // This file's first update
        0
    } else {
        // Not our first propagated update to this lower archive
        let distance = (point.interval - base.interval) / archive.seconds_per_point;
        distance % archive.points
    };

    fh.seek(io::SeekFrom::Start((archive.offset + index * point_size).into()))?;
    point.write(fh)?;

    Ok(())
}

fn write_archive<F: Write + Seek>(fh: &mut F, archive: &ArchiveInfo, points: &[Point], base_interval: u32) -> Result<(), io::Error> {
    let point_size = 12;

    let first_interval = points[0].interval;
    let point_distance = (first_interval - base_interval) / archive.seconds_per_point;
    let offset = point_distance % archive.points;

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

    let higher_first_index =
        if higher_base.interval == 0 {
            0
        } else {
            let time_distance = lower_interval_start - higher_base.interval;
            let point_distance = time_distance / higher.seconds_per_point;
            point_distance % higher.points
        };

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

/**
 * update(path, value, timestamp=None)
 *
 * path is a string
 * value is a float
 * timestamp is either an int or float
 */
pub fn update(path: &Path, value: f64, timestamp: u32, now: u32) -> Result<(), io::Error> {
    let mut fh = fs::OpenOptions::new().read(true).write(true).open(path)?;
    // if CAN_FADVISE and FADVISE_RANDOM:
    //     posix_fadvise(fh.fileno(), 0, 0, POSIX_FADV_RANDOM)

    file_update(&mut fh, value, timestamp, now)
}

fn file_update(fh: &mut fs::File, value: f64, timestamp: u32, now: u32) -> Result<(), io::Error> {
    // if LOCK:
    //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

    let header = __read_header(fh)?;

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
    let point = Point { interval, value };

    write_archive_point(fh, archive, &point)?;

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

/**
 * update_many(path,points)
 *
 * path is a string
 * points is a list of (timestamp,value) points
 */
pub fn update_many(path: &Path, points: &[Point], now: u32) -> Result<(), io::Error> {
    if points.is_empty() {
        return Ok(());
    }

    let mut points_vec = points.to_vec();
    points_vec.sort_by_key(|p| std::u32::MAX - p.interval); // Order points by timestamp, newest first

    let mut fh = fs::OpenOptions::new().read(true).write(true).open(path)?;
    // if CAN_FADVISE and FADVISE_RANDOM:
    //     posix_fadvise(fh.fileno(), 0, 0, POSIX_FADV_RANDOM)

    file_update_many(&mut fh, &points_vec, now)
}

fn file_update_many(fh: &mut fs::File, points: &[Point], now: u32) -> Result<(), io::Error> {
    // if LOCK:
    //     fcntl.flock(fh.fileno(), fcntl.LOCK_EX)

    let header = __read_header(fh)?;
    let mut archive_index = 0;
    let mut current_points = vec![];

    for point in points {
        let age = now - point.interval;

        while header.archives[archive_index].retention() < age {  // We can't fit any more points in this archive
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

pub fn info(path: &Path) -> Result<WhisperMetadata, io::Error> {
    let mut fh = fs::File::open(path)?;
    let info = __read_header(&mut fh)?;
    Ok(info)
}

pub struct ArchiveData {
    pub from_interval: u32,
    pub until_interval: u32,
    pub step: u32,
    pub values: Vec<Option<f64>>,
}

impl ArchiveData {
    pub fn points(&self) -> Vec<Point> {
        range_step(self.from_interval, self.until_interval, self.step)
            .zip(&self.values)
            .filter_map(|(interval, value)| value.map(|value| Point { interval, value }))
            .collect()
    }
}

fn available_interval(header: &WhisperMetadata, now: u32) -> Result<Interval, String> {
    Interval::new(now - header.max_retention, now)
}

fn available_interval_ai(ai: &ArchiveInfo, now: u32) -> Interval {
    Interval::new(now - ai.retention(), now).unwrap()
}

pub fn suggest_archive(header: &WhisperMetadata, interval: Interval, now: u32) -> Option<&ArchiveInfo> {
    let available = available_interval(header, now).ok()?;
    let adjusted = available.intersection(interval).ok()?;
    header.archives.iter().find(|archive| available_interval_ai(archive, now).contains(adjusted))
}

pub fn find_archive(header: &WhisperMetadata, seconds_per_point: u32) -> Result<&ArchiveInfo, String> {
    for archive in &header.archives {
        if archive.seconds_per_point == seconds_per_point {
            return Ok(archive);
        }
    }
    Err(format!("Invalid granularity: {}", seconds_per_point))
}

/**
 * fetch(path,fromTime,untilTime=None,archiveToSelect=None)
 *
 * path is a string
 * fromTime is an epoch time
 * untilTime is also an epoch time, but defaults to now.
 * archiveToSelect is the requested granularity, but defaults to None.
 *
 * Returns a tuple of (timeInfo, valueList)
 * where timeInfo is itself a tuple of (fromTime, untilTime, step)
 *
 * Returns None if no data can be returned
 */
pub fn fetch(path: &Path, interval: Interval, now: u32, seconds_per_point: u32) -> Result<Option<ArchiveData>, io::Error> {
    // if now is None:
    //     now = int(time.time())
    // if untilTime is None:
    //     untilTime = now

    let mut fh = fs::OpenOptions::new().read(true).open(path)?;

    let header = __read_header(&mut fh)?;

    let archive = find_archive(&header, seconds_per_point)
        .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

    file_fetch(&mut fh, &header, &archive, interval, now)
}

fn file_fetch<F: Read + Seek>(fh: &mut F, header: &WhisperMetadata, archive: &ArchiveInfo, interval: Interval, now: u32) -> Result<Option<ArchiveData>, io::Error> {
    let available = available_interval(&header, now)
        .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

    if !interval.intersects(available) {
        // Range is in the future or beyond retention
        return Ok(None);
    }

    let interval = available_interval(&header, now)
        .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?
        .intersection(interval)
        .map_err(|s| io::Error::new(io::ErrorKind::Other, s))?;

    __archive_fetch(fh, archive, interval.from(), interval.until()).map(Some)
}

/**
 * Fetch data from a single archive. Note that checks for validity of the time
 * period requested happen above this level so it's possible to wrap around the
 * archive on a read and request data older than the archive's retention
 */
fn __archive_fetch<R: Read + Seek>(fh: &mut R, archive: &ArchiveInfo, from_time: u32, until_time: u32) -> Result<ArchiveData, io::Error> {
    let step = archive.seconds_per_point;
    let from_interval = (from_time - (from_time % step)) + step;
    let mut until_interval = (until_time - (until_time % step)) + step;

    if from_interval == until_interval {
        // Zero-length time range: always include the next point
        until_interval += step;
    }

    fh.seek(io::SeekFrom::Start(archive.offset.into()))?;
    let base = Point::read(fh)?;

    if base.interval == 0 {
        let points = (until_interval - from_interval) / step;
        let mut values = Vec::new();
        for _i in 0..points {
            values.push(None);
        }
        return Ok(ArchiveData {
            from_interval,
            until_interval,
            step,
            values,
        })
    }

    // Determine from_index
    let point_distance = from_interval.checked_sub(base.interval).unwrap_or(0) / step;
    let from_index = point_distance % archive.points;

    // Determine until_index
    let point_distance = until_interval.checked_sub(base.interval).unwrap_or(0) / step;
    let until_index = point_distance % archive.points;

    let series = read_archive(fh, &archive, from_index, until_index)?;

    // And finally we construct a list of values
    let values = points_to_values(&series, from_interval, step);

    Ok(ArchiveData { from_interval, until_interval, step, values })
}

/**
 * Merges the data from one whisper file into another. Each file must have
 * the same archive configuration. time_from and time_to can optionally be
 * specified for the merge.
 */
pub fn merge(path_src: &Path, path_dst: &Path, time_from: u32, time_to: u32, now: u32) -> Result<(), io::Error> {
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

    let mut fh_src = fs::OpenOptions::new().read(true).open(path_src)?;
    let mut fh_dst = fs::OpenOptions::new().read(true).write(true).open(path_dst)?;
    file_merge(&mut fh_src, &mut fh_dst, time_from, time_to, now)
}

fn file_merge<F1: Read + Seek, F2: Read + Write + Seek>(fh_src: &mut F1, fh_dst: &mut F2, time_from: u32, time_to: u32, now: u32) -> Result<(), io::Error> {
    let header_src = __read_header(fh_src)?;
    let header_dst = __read_header(fh_dst)?;
    if header_src.archives != header_dst.archives {
        return Err(io::Error::new(io::ErrorKind::Other, "Archive configurations are unalike. Resize the input before merging"));
    }

    // Sanity check: do not mix the from/to values.
    if time_to < time_from {
        return Err(io::Error::new(io::ErrorKind::Other, "time_to must be >= time_from"));
    }

    let mut archives = header_src.archives.clone();
    archives.sort_by_key(|archive| archive.retention());

    for (index, archive) in archives.iter().enumerate() {
        // if time_to is too old, skip this archive
        if time_to < now - archive.retention() {
            continue;
        }

        let from = u32::max(time_from, now - archive.retention());
        let data = __archive_fetch(fh_src, &archive, from, time_to)?;

        let points = data.points();
        if !points.is_empty() {
            __archive_update_many(fh_dst, &header_dst, index, &points)?;
        }
    }
    Ok(())
}

pub struct DiffPoint {
    pub interval: u32,
    pub value1: Option<f64>,
    pub value2: Option<f64>,
}

pub struct DiffArchive {
    pub index: usize,
    pub diffs: Vec<DiffPoint>,
    pub total: usize,
}

/** Compare two whisper databases. Each file must have the same archive configuration */
pub fn diff(path1: &Path, path2: &Path, ignore_empty: bool, until_time: u32, now: u32) -> Result<Vec<DiffArchive>, io::Error> {
    // if now is None:
    //     now = int(time.time())
    // if until_time:
    //     untilTime = until_time
    // else:
    //     untilTime = now

    let mut fh1 = fs::File::open(path1)?;
    let mut fh2 = fs::File::open(path2)?;

    file_diff(&mut fh1, &mut fh2, ignore_empty, until_time, now)
}

fn file_diff<F1: Read + Seek, F2: Read + Seek>(fh1: &mut F1, fh2: &mut F2, ignore_empty: bool, mut until_time: u32, now: u32) -> Result<Vec<DiffArchive>, io::Error> {
    let metadata1 = WhisperMetadata::read(fh1)?;
    let metadata2 = WhisperMetadata::read(fh2)?;

    if metadata1.archives != metadata2.archives {
        return Err(io::Error::new(io::ErrorKind::Other, "Archive configurations are unalike. Resize the input before diffing"));
    }

    let mut archives = metadata1.archives.clone();
    archives.sort_by_key(|a| a.retention());

    let mut archive_diffs = Vec::new();

    for (index, archive) in archives.iter().enumerate() {
        let start_time = now - archive.retention();
        let data1 = __archive_fetch(fh1, archive, start_time, until_time)?;
        let data2 = __archive_fetch(fh2, archive, start_time, until_time)?;

        let start = u32::min(data1.from_interval, data2.from_interval);
        let end = u32::max(data1.until_interval, data2.until_interval);
        let archive_step = u32::min(data1.step, data2.step);

        let points: Vec<DiffPoint> = range_step(start, end, archive_step)
            .enumerate()
            .map(|(index, interval)| DiffPoint {
                interval,
                value1: data1.values[index],
                value2: data2.values[index],
            })
            .filter(|p|
                if ignore_empty {
                    p.value1.is_some() && p.value2.is_some()
                } else {
                    p.value1.is_some() || p.value2.is_some()
                }
            )
            .collect();

        let total = points.len();

        let diffs = points.into_iter().filter(|p| p.value1 != p.value2).collect();

        archive_diffs.push(DiffArchive { index, diffs, total });

        until_time = u32::min(start_time, until_time);
    }

    Ok(archive_diffs)
}
