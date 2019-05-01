use config::*;
use serde::*;
use std::path::PathBuf;

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
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::from_str(
            include_str!("config.toml"),
            FileFormat::Toml,
        ))?;

        s.try_into()
    }
}
