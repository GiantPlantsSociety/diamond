use config::*;
use serde::*;
use std::fs;
use std::path::PathBuf;

const CONFIG: &str = include_str!("config.toml");

#[derive(Debug, Deserialize)]
pub struct Tcp {
    pub port: u32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db_path: PathBuf,
    pub tcp: Tcp,
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
