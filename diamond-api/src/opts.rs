use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "diamond-api")]
pub struct Args {
    /// Path to data directory, default value is a current directory
    #[structopt(
        name = "path",
        short = "d",
        long = "data-dir",
        default_value = ".",
        parse(from_os_str)
    )]
    pub path: PathBuf,

    /// Force to create data directory if it is absent
    #[structopt(name = "force", short = "f", long = "force")]
    pub force: bool,

    /// Port to listen on
    #[structopt(name = "port", short = "p", long = "port", default_value = "8080")]
    pub port: u16,
}
