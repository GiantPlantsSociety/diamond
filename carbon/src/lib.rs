#[macro_use]
extern crate failure;

use failure::Error;
use std::convert::From;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub timestamp: u32,
    pub value: f64,
}

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

impl FromStr for MetricPoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<MetricPoint, Error> {
        let segments: Vec<&str> = s.split(' ').collect();

        let (name, timestamp, value) = match segments.len() {
            3 => (segments[0], segments[1], segments[2]),
            _ => return Err(format_err!("can not parse metric from line: {}", s)),
        };

        Ok(MetricPoint {
            name: name.to_owned(),
            point: Point {
                timestamp: timestamp.parse()?,
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
                name: name.clone(),
                point: point.clone(),
            });
        }
        vector
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricPath(pub PathBuf);

impl FromStr for MetricPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<MetricPath, Error> {
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
    use metrics::{MetricPath, MetricPoint, Point};
    use std::path::Path;

    #[test]
    fn metrics_correct_parse() {
        let metric_result = "this.is.correct 1 123".parse::<MetricPoint>();
        assert!(metric_result.is_ok(), "It should be parsed");

        let metric = metric_result.unwrap();
        assert_eq!(
            metric,
            MetricPoint {
                name: "this.is.correct".to_owned(),
                point: Point {
                    timestamp: 1,
                    value: 123_f32,
                }
            },
            "It should be matched"
        );
    }
    #[test]
    fn metric_path_correct_parse() {
        let metric_result = "this.is.correct.path".parse::<MetricPath>();
        assert!(metric_result.is_ok(), "It should be parsed");

        let metric = metric_result.unwrap();
        assert_eq!(
            metric,
            Path::new("./this/is/correct/path.wsp").to_owned().into(),
            "It should be matched"
        );
    }

}
