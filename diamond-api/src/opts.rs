use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    /// Path to data directory, default value is a current directory
    #[arg(name = "path", short = 'd', long = "data-dir", default_value = ".")]
    pub path: PathBuf,

    /// Force to create data directory if it is absent
    #[arg(name = "force", short = 'f', long = "force")]
    pub force: bool,

    /// Port to listen on
    #[arg(name = "port", short = 'p', long = "port", default_value = "8080")]
    pub port: u16,
}
