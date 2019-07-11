use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "diamond-api")]
pub struct Args {
    /// Path to data dir, default value is current dir
    #[structopt(name = "path", short = "d", long = "data-dir", parse(from_os_str))]
    pub path: PathBuf,

    /// Path to data dir, default value is current dir
    #[structopt(name = "force", short = "f", long = "force")]
    pub force: bool,

    /// Port to listen on
    #[structopt(name = "port", short = "p", long = "port", default_value = "8080")]
    pub port: u16,
}
