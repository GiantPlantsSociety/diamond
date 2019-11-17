use failure::*;
use lazy_static::lazy_static;

use regex::Regex;
use std::convert::From;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::builder::WhisperBuilder;
use whisper::point::Point;
use whisper::WhisperFile;

pub mod settings;

use settings::Settings;
use settings::WhisperConfig;

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
    #[fail(display = "Metric line({}) cannot be validated", _0)]
    Validate(String),
    #[fail(display = "Metric name({}) cannot be validated", _0)]
    NameValidate(String),
    #[fail(display = "Cannot parse metric from line: {}", _0)]
    LineParse(String),
}

impl MetricPoint {
    fn validate(s: &str) -> Result<(), MetricError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[\d\w\._ -]+$").unwrap();
        }

        if RE.is_match(s) {
            Ok(())
        } else {
            Err(MetricError::Validate(s.to_owned()))
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
            _ => return Err(MetricError::LineParse(s.to_owned()).into()),
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
    fn from(mp: MetricPoints) -> Vec<MetricPoint> {
        let name = mp.name;
        mp.points
            .into_iter()
            .map(|point| MetricPoint {
                name: name.clone(),
                point,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricPath(PathBuf);

impl MetricPath {
    fn validate(s: &str) -> Result<(), MetricError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[\d\w\._-]+$").unwrap();
        }

        if RE.is_match(s) {
            Ok(())
        } else {
            Err(MetricError::NameValidate(s.to_owned()))
        }
    }
}

impl FromStr for MetricPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
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
    fn from(metric_path: MetricPath) -> Self {
        metric_path.0
    }
}

impl From<PathBuf> for MetricPath {
    fn from(metric_path: PathBuf) -> Self {
        MetricPath(metric_path)
    }
}

#[inline]
pub fn line_update<P: AsRef<Path>>(
    message: &str,
    dir: P,
    config: &WhisperConfig,
    now: u32,
) -> Result<(), Error> {
    let metric: MetricPoint = message.parse()?;
    let metric_path: MetricPath = metric.name.parse()?;

    let file_path = dir.as_ref().join(metric_path.0);

    let mut file = if file_path.exists() {
        WhisperFile::open(&file_path)?
    } else {
        let dir_path = file_path.parent().unwrap();
        fs::create_dir_all(&dir_path)?;

        WhisperBuilder::default()
            .add_retentions(&config.retentions)
            .x_files_factor(config.x_files_factor)
            .aggregation_method(config.aggregation_method)
            .build(&file_path)?
    };

    file.update(&metric.point, now)?;

    Ok(())
}

#[inline]
pub fn update_silently(line: &str, conf: &Settings) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    line_update(&line, &conf.db_path, &conf.whisper, now).unwrap_or_else(|e| eprintln!("{}", e));
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings::{Net, WhisperConfig};
    use std::convert::From;
    use std::net::IpAddr::V4;
    use std::path::{Path, PathBuf};
    use tempfile::Builder;
    use whisper::aggregation::AggregationMethod;
    use whisper::retention::Retention;

    #[test]
    fn metric_path_validate_ok() {
        assert!(MetricPath::validate("this.is.correct").is_ok());
    }

    #[test]
    fn metric_path_validate_err() {
        assert!(MetricPath::validate("$this.is.not.correct").is_err());
        assert!(MetricPath::validate(",this.is.not.correct").is_err());
        assert!(MetricPath::validate("@this.is.not.correct").is_err());
        assert!(MetricPath::validate("\\this.is.not.correct").is_err());
        assert!(MetricPath::validate("/this.is.not.correct").is_err());
        assert!(MetricPath::validate("[this.is.not.correct").is_err());
        assert!(MetricPath::validate("{this.is.not.correct").is_err());
        assert!(MetricPath::validate("%this.is.not.correct").is_err());
        assert!(MetricPath::validate("#this.is.not.correct").is_err());
    }

    #[test]
    fn metric_path_conversion_ok() {
        let m: PathBuf = "this.is.ok".parse::<MetricPath>().unwrap().into();
        assert_eq!(PathBuf::from("./this/is/ok.wsp"), m);
    }

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
            metric_result
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
    fn test_metric_parse_incorrect_parts_count() {
        let metric_result = "this.is.correct 1 123 1".parse::<MetricPoint>();
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
            metric_result
        );
    }

    #[test]
    fn test_metrics_points_to_vec_metric_point() {
        let p1 = Point {
            interval: 100,
            value: 100.1,
        };
        let p2 = Point {
            interval: 200,
            value: 100.2,
        };
        let p3 = Point {
            interval: 300,
            value: 100.3,
        };
        let name = String::from("test-metric-name");
        let mp = MetricPoints {
            name: name.clone(),
            points: vec![p1, p2, p3],
        };
        let points_vec: Vec<MetricPoint> = mp.into();
        assert_eq!(
            points_vec,
            vec![
                MetricPoint {
                    name: name.clone(),
                    point: p1
                },
                MetricPoint {
                    name: name.clone(),
                    point: p2
                },
                MetricPoint {
                    name: name.clone(),
                    point: p3
                },
            ]
        );
    }

    #[test]
    fn update_line_with_absent_wsp() -> Result<(), Error> {
        let dir = Builder::new()
            .prefix("diamond")
            .tempdir()
            .unwrap()
            .path()
            .to_path_buf();

        let message = "this.is.correct1 1545778338 124";

        let config = WhisperConfig {
            retentions: vec![Retention {
                seconds_per_point: 1,
                points: 1000,
            }],
            x_files_factor: 0.5,
            aggregation_method: AggregationMethod::Average,
        };
        let now = 1_545_778_348;
        line_update(message, &dir, &config, now)?;

        let file = dir.join("this").join("is").join("correct1.wsp");
        assert_eq!(
            WhisperFile::open(&file)?.dump(1)?[0],
            Point {
                interval: now - 10,
                value: 124.0
            }
        );

        Ok(())
    }

    #[test]
    fn update_line_with_present_wsp() -> Result<(), Error> {
        let dir = Builder::new()
            .prefix("diamond")
            .tempdir()
            .unwrap()
            .path()
            .to_path_buf();

        let file_path = dir.join("this").join("is").join("correct2.wsp");

        fs::create_dir_all(&file_path.parent().unwrap())?;

        let mut file = WhisperBuilder::default()
            .add_retentions(&[Retention {
                seconds_per_point: 1,
                points: 20,
            }])
            .x_files_factor(0.5)
            .aggregation_method(AggregationMethod::Average)
            .build(&file_path)?;

        let message = "this.is.correct2 1545778338 123";

        let config = WhisperConfig {
            retentions: vec![Retention {
                seconds_per_point: 1,
                points: 20,
            }],
            x_files_factor: 0.5,
            aggregation_method: AggregationMethod::Average,
        };
        let now = 1_545_778_348;
        line_update(message, &dir, &config, now)?;

        assert_eq!(
            file.dump(1)?[0],
            Point {
                interval: now - 10,
                value: 123.0
            }
        );

        Ok(())
    }

    #[test]
    fn update_silently_with_absent_wsp() -> Result<(), Error> {
        let dir = Builder::new()
            .prefix("diamond_silent")
            .tempdir()
            .unwrap()
            .path()
            .to_path_buf();

        let config = Settings {
            db_path: dir.clone(),
            tcp: Net {
                port: 6142,
                host: V4("0.0.0.0".parse().unwrap()),
            },
            udp: Net {
                port: 6142,
                host: V4("0.0.0.0".parse().unwrap()),
            },
            whisper: WhisperConfig {
                x_files_factor: 0.5,
                retentions: vec![Retention {
                    seconds_per_point: 1,
                    points: 1000,
                }],
                aggregation_method: AggregationMethod::Average,
            },
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            - 10;

        let message = format!("this.is.correct1 {} 124", timestamp);

        update_silently(&message, &config);

        let file = dir.join("this").join("is").join("correct1.wsp");
        assert_eq!(
            WhisperFile::open(&file)?.dump(1)?[0],
            Point {
                interval: timestamp,
                value: 124.0
            }
        );

        Ok(())
    }
}
