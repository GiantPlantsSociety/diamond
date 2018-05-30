#[macro_use] extern crate structopt;
use structopt::StructOpt;
use std::process::exit;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-info")]
struct Args {
    /// Outputs results in JSON form
    #[structopt(long = "json")]
    json: bool,
    /// Path to data file
    #[structopt(name = "path", parse(from_os_str))]
    path: PathBuf,
    /// File info field to display
    field: Option<String>,
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
//
// Archive 0
// retention: 86400
// secondsPerPoint: 60
// points: 1440
// size: 17280
// offset: 28
//

// whisper-info.py load.1m.wsp size
// Unknown field "size". Valid fields are maxRetention,xFilesFactor,aggregationMethod,archives,fileSize

// whisper-info.py load.1m.wsp fileSize
// 17308

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

// whisper-info.py load.2m.wsp
// maxRetention: 172800
// xFilesFactor: 0.5
// aggregationMethod: average
// fileSize: 34600
//
// Archive 0
// retention: 86400
// secondsPerPoint: 60
// points: 1440
// size: 17280
// offset: 40
//
// Archive 1
// retention: 172800
// secondsPerPoint: 120
// points: 1440
// size: 17280
// offset: 17320
//

fn run(args: &Args) -> Result<(), String> {
    println!("whisper-info {}", env!("CARGO_PKG_VERSION"));
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
