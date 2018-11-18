extern crate failure;
extern crate lazy_static;
extern crate regex;
extern crate whisper;

use failure::Error;
use failure::*;
use lazy_static::lazy_static;

use regex::Regex;
use std::convert::From;
use std::path::PathBuf;
use std::str::FromStr;
use whisper::point::Point;

#[derive(Debug, Clone, PartialEq)]
pub struct MetricPoint {
    pub name: String,
    pub point: Point,
}

#[derive(Debug, Clone)]
pub struct MetricPoints {
    pub name: String,
    pub points: Vec<Point>,
}

#[derive(Fail, Debug)]
enum MetricError {
    #[fail(display = "Metric line({}) can not be validated", _0)]
    ValidateError(String),
    #[fail(display = "Metric name({}) can not be validated", _0)]
    NameValidateError(String),
    #[fail(display = "Can not parse metric from line: {}", _0)]
    MetricLineParseError(String),
}

impl MetricPoint {
    fn validate(s: &str) -> Result<(), MetricError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[\d\w\._ -]+$").unwrap();
        }

        if RE.is_match(s) {
            Ok(())
        } else {
            Err(MetricError::ValidateError(s.to_owned()))
        }
    }
}

impl FromStr for MetricPoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<MetricPoint, Error> {
        MetricPoint::validate(s)?;

        let segments: Vec<&str> = s.split(' ').collect();

        let (name, timestamp, value) = match segments.len() {
            3 => (segments[0], segments[1], segments[2]),
            _ => return Err(MetricError::MetricLineParseError(s.to_owned()).into()),
        };

        Ok(MetricPoint {
            name: name.to_owned(),
            point: Point {
                interval: timestamp.parse()?,
                value: value.parse()?,
            },
        })
    }
}

impl From<MetricPoints> for Vec<MetricPoint> {
    fn from(m: MetricPoints) -> Vec<MetricPoint> {
        let mut vector: Vec<MetricPoint> = Vec::new();
        let name = m.name;
        for point in m.points.iter() {
            vector.push(MetricPoint {
                name: name.to_owned(),
                point: point.to_owned(),
            });
        }
        vector
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricPath(pub PathBuf);

impl MetricPath {
    fn validate(s: &str) -> Result<(), MetricError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[\d\w\._-]+$").unwrap();
        }

        if RE.is_match(s) {
            Ok(())
        } else {
            Err(MetricError::NameValidateError(s.to_owned()))
        }
    }
}

impl FromStr for MetricPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<MetricPath, Error> {
        MetricPath::validate(s)?;
        let segments: Vec<&str> = s.split('.').collect();

        let path = segments
            .iter()
            .fold(PathBuf::new().join("."), |acc, x| acc.join(x))
            .with_extension("wsp");

        Ok(MetricPath(path))
    }
}

impl From<MetricPath> for PathBuf {
    fn from(metric_path: MetricPath) -> PathBuf {
        metric_path.0
    }
}

impl From<PathBuf> for MetricPath {
    fn from(metric_path: PathBuf) -> MetricPath {
        MetricPath(metric_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_metric_correct_parse() {
        let metric_result = "this.is.correct 1 123".parse::<MetricPoint>();
        assert!(metric_result.is_ok(), "It should be parsed");

        let metric = metric_result.unwrap();
        assert_eq!(
            metric,
            MetricPoint {
                name: "this.is.correct".to_owned(),
                point: Point {
                    interval: 1,
                    value: 123_f64,
                }
            },
            "It should be matched"
        );
    }

    #[test]
    fn test_metric_parse_incorrect_name() {
        let s = "this\\.is./incorrect 1 123";
        let metric_result = s.parse::<MetricPoint>();
        assert!(
            metric_result.is_err(),
            "It({}) should not be parsed {:?}",
            s,
            metric_result.unwrap()
        );
    }

    #[test]
    fn test_metric_parse_incorrect_time() {
        let metric_result = "this.is.correct a 123".parse::<MetricPoint>();
        assert!(metric_result.is_err(), "It should not be parsed");
    }

    #[test]
    fn test_metric_parse_incorrect_value() {
        let metric_result = "this.is.correct 1 a123".parse::<MetricPoint>();
        assert!(metric_result.is_err(), "It should not be parsed");
    }

    #[test]
    fn test_metric_path_parse_correct() {
        let metric_result = "this.is.correct.path".parse::<MetricPath>();
        assert!(metric_result.is_ok(), "It should be parsed");

        let metric = metric_result.unwrap();
        assert_eq!(
            metric,
            Path::new("./this/is/correct/path.wsp").to_owned().into(),
            "It should be matched"
        );
    }

    #[test]
    fn test_metric_path_parse_incorrect() {
        let s = "this/.is.incorrect.path";
        let metric_result = s.parse::<MetricPath>();
        assert!(
            metric_result.is_err(),
            "It({}) should not be parsed {:?}",
            s,
            metric_result.unwrap()
        );
    }
}
