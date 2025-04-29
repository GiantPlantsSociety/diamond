use super::*;
use crate::aggregation::AggregationMethod;
use crate::retention::Retention;
use std::convert::AsRef;
use std::default;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;

pub struct WhisperBuilder {
    aggregation_method: AggregationMethod,
    x_files_factor: f32,
    retentions: Vec<Retention>,
    sparse: bool,
}

impl default::Default for WhisperBuilder {
    fn default() -> Self {
        Self {
            aggregation_method: AggregationMethod::Average,
            x_files_factor: 0.5,
            retentions: Vec::new(),
            sparse: false,
        }
    }
}

impl WhisperBuilder {
    pub fn add_retentions(mut self, retentions: &[Retention]) -> Self {
        self.retentions.extend(retentions);
        self
    }

    pub fn add_retention(mut self, retention: Retention) -> Self {
        self.retentions.push(retention);
        self
    }

    pub fn aggregation_method(mut self, aggregation_method: AggregationMethod) -> Self {
        self.aggregation_method = aggregation_method;
        self
    }

    pub fn x_files_factor(mut self, x_files_factor: f32) -> Self {
        self.x_files_factor = x_files_factor;
        self
    }

    pub fn sparse(mut self, sparse: bool) -> Self {
        self.sparse = sparse;
        self
    }

    fn into_metadata(mut self) -> Result<WhisperMetadata, BuilderError> {
        if self.x_files_factor < 0.0 || self.x_files_factor > 1.0 {
            return Err(BuilderError::InvalidXFilesFactor(self.x_files_factor));
        }

        if self.retentions.is_empty() {
            return Err(BuilderError::NoRetentions);
        }

        self.retentions.sort_by_key(|a| a.seconds_per_point);
        validate_archive_list(&self.retentions)?;

        let mut archives = Vec::with_capacity(self.retentions.len());
        let mut offset = METADATA_SIZE + ARCHIVE_INFO_SIZE * self.retentions.len();
        for retention in &self.retentions {
            archives.push(ArchiveInfo {
                offset: offset as u32,
                seconds_per_point: retention.seconds_per_point,
                points: retention.points,
            });
            offset += retention.points as usize * POINT_SIZE;
        }

        let max_retention = archives
            .iter()
            .map(|archive| archive.retention())
            .max()
            .unwrap();

        let metadata = WhisperMetadata {
            aggregation_method: self.aggregation_method,
            max_retention,
            x_files_factor: self.x_files_factor,
            archives,
        };

        Ok(metadata)
    }

    pub fn build<P: AsRef<Path>>(self, path: P) -> Result<WhisperFile, BuilderError> {
        let sparse = self.sparse;
        let metadata = self.into_metadata()?;
        let file =
            WhisperFile::create(&metadata, path.as_ref(), sparse).map_err(BuilderError::Io)?;
        Ok(file)
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
fn validate_archive_list(archives: &[Retention]) -> Result<(), BuilderError> {
    for (i, pair) in archives.windows(2).enumerate() {
        let archive = &pair[0];
        let next_archive = &pair[1];

        if archive.seconds_per_point >= next_archive.seconds_per_point {
            return Err(BuilderError::SamePrecision(i, *archive, *next_archive));
        }

        if next_archive.seconds_per_point % archive.seconds_per_point != 0 {
            return Err(BuilderError::UndividablePrecision(
                i,
                *archive,
                *next_archive,
            ));
        }

        let retention = archive.retention();
        let next_retention = next_archive.retention();

        if next_retention <= retention {
            return Err(BuilderError::BadRetention(i, retention, next_retention));
        }

        let points_per_consolidation = next_archive.seconds_per_point / archive.seconds_per_point;
        if archive.points < points_per_consolidation {
            return Err(BuilderError::NotEnoughPoints(
                i + 1,
                points_per_consolidation,
                archive.points,
            ));
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum BuilderError {
    NoRetentions,
    SamePrecision(usize, Retention, Retention),
    UndividablePrecision(usize, Retention, Retention),
    BadRetention(usize, u32, u32),
    NotEnoughPoints(usize, u32, u32),
    InvalidXFilesFactor(f32),
    Io(io::Error),
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::NoRetentions => write!(f, "You must specify at least one retention"),
            Self::SamePrecision(size, precision, precision2) => write!(
                f,
                "A Whisper database may not be configured having two archives with the same precision (at index {}, {:?} and next is {:?})",
                size, precision, precision2
            ),
            Self::UndividablePrecision(size, precision, precision2) => write!(
                f,
                "Higher precision archives' precision must evenly divide all lower precision archives' precision (at index {}, {:?} and next is {:?})",
                size, precision, precision2
            ),
            Self::BadRetention(index, retention, retention2) => write!(
                f,
                "Lower precision archives must cover larger time intervals than higher precision archives (at index {}: {} seconds and next is {} seconds)",
                index, retention, retention2
            ),
            Self::NotEnoughPoints(index, points, points2) => write!(
                f,
                "Each archive must have at least enough points to consolidate to the next archive (archive at index {} consolidates {} of previous archive's points but it has only {} total points)",
                index, points, points2
            ),
            Self::InvalidXFilesFactor(factor) => {
                write!(f, "Invalid xFilesFactor {}, not between 0 and 1", factor)
            }
            Self::Io(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<BuilderError> for error::Error {
    fn from(error: BuilderError) -> Self {
        match error {
            BuilderError::Io(e) => error::Error::Io(e),
            e => error::Error::Kind(e.to_string()),
        }
    }
}
