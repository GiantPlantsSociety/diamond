use std::env;
use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
#[derive(Debug)]
struct Tcp {
    port: u32
}