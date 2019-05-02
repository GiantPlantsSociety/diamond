use config::*;
use serde::*;
use std::convert::From;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use whisper::aggregation::AggregationMethod;
use whisper::retention;

const CONFIG: &str = include_str!("config.toml");

#[derive(Debug, Deserialize)]
pub struct Tcp {
    pub port: u32,
    pub host: IpAddr,
}

#[derive(Debug, Deserialize)]
pub struct Retention(u32, u32);

impl From<Retention> for retention::Retention {
    fn from(r: Retention) -> Self {
        retention::Retention {
            seconds_per_point: r.0,
            points: r.1,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WhisperConfig {
    pub x_files_factor: f32,
    pub retentions: Vec<Retention>,
    pub aggregation_method: AggregationMethod,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db_path: PathBuf,
    pub tcp: Tcp,
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

    pub fn generate(path: PathBuf) -> Result<(), std::io::Error> {
        fs::write(path, CONFIG)
    }
}
