#[macro_use] extern crate structopt;
use structopt::StructOpt;
use std::process::exit;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-info")]
struct Args {
    #[structopt(long = "json")]
    json: bool,
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
}

// whisper-info.py 
// Usage: whisper-info.py [options] path [field]

// Options:
//   -h, --help  show this help message and exit
//   --json      Output results in JSON form

// whisper-info.py load.1m.wsp
// maxRetention: 86400
// xFilesFactor: 0.5
// aggregationMethod: average
// fileSize: 17308

// Archive 0
// retention: 86400
// secondsPerPoint: 60
// points: 1440
// size: 17280
// offset: 28

// whisper-info.py load.1m.wsp --json
// {
//   "maxRetention": 86400,
//   "xFilesFactor": 0.5,
//   "aggregationMethod": "average",
//   "archives": [
//     {
//       "retention": 86400,
//       "secondsPerPoint": 60,
//       "points": 1440,
//       "size": 17280,
//       "offset": 28
//     }
//   ],
//   "fileSize": 17308
// }

fn run(args: &Args) -> Result<(), String> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("whisper-info");
    println!("{:?}", args);
    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
