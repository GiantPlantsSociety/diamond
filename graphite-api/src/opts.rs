use std::path::PathBuf;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "graphite-api")]
pub struct Args {
    /// Path to data dir, default value is current dir
    #[structopt(name = "path", short = "d", long = "data-dir", parse(from_os_str))]
    pub path: PathBuf,

    /// Path to data dir, default value is current dir
    #[structopt(name = "force", short = "f", long = "force")]
    pub force: bool,
}
