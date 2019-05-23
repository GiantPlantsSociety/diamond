use config::*;
use serde::*;
use std::convert::From;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use whisper::aggregation::AggregationMethod;
use whisper::retention::Retention;

const CONFIG: &str = include_str!("config.toml");

#[derive(Debug, PartialEq, Deserialize)]
pub struct Net {
    pub port: u32,
    pub host: IpAddr,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct WhisperConfig {
    pub x_files_factor: f32,
    pub retentions: Vec<Retention>,
    pub aggregation_method: AggregationMethod,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Settings {
    pub db_path: PathBuf,
    pub tcp: Net,
    pub udp: Net,
    pub whisper: WhisperConfig,
}

impl Settings {
    pub fn new(file: Option<PathBuf>) -> Result<Self, ConfigError> {
        let mut s = Config::default();

        match file {
            Some(file) => s
                .merge(File::from_str(CONFIG, FileFormat::Toml))?
                .merge(File::from(file))?,
            _ => s.merge(File::from_str(CONFIG, FileFormat::Toml))?,
        };

        s.try_into()
    }

    pub fn generate<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
        fs::write(path, CONFIG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use std::net::IpAddr::V4;
    use tempfile::Builder;
    use whisper::retention::Retention;

    #[test]
    fn test_default_config() {
        let default_config = Settings::new(None).unwrap();

        let etalon = Settings {
            db_path: PathBuf::from("/var/db/diamond"),
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
                    seconds_per_point: 60,
                    points: 1440,
                }],
                aggregation_method: AggregationMethod::Average,
            },
        };

        assert_eq!(default_config, etalon);
    }

    #[test]
    fn test_config_load() {
        let path = Builder::new()
            .prefix("carbon")
            .suffix("config.toml")
            .tempfile()
            .unwrap()
            .path()
            .to_path_buf();

        let s = "db_path = \"/tmp\"";
        fs::write(&path, s).unwrap();

        let config = Settings::new(Some(path)).unwrap();

        let etalon = Settings {
            db_path: PathBuf::from("/tmp/"),
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
                    seconds_per_point: 60,
                    points: 1440,
                }],
                aggregation_method: AggregationMethod::Average,
            },
        };

        assert_eq!(config, etalon);
    }

    #[test]
    fn test_generate_config() {
        let path = Builder::new()
            .prefix("carbon")
            .suffix("config.toml")
            .tempfile()
            .unwrap()
            .path()
            .to_path_buf();

        Settings::generate(&path).unwrap();
        let s = read_to_string(&path).unwrap();

        assert_eq!(s, CONFIG);
    }
}
