#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

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

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    println!("whisper-info {}", env!("CARGO_PKG_VERSION"));
    println!("{:?}", args);

    let file = whisper::WhisperFile::open(&args.path)?;
    let meta = file.info();

    let info = match &args.field {
        Some(ref field) if field == "maxRetention" =>  meta.max_retention.to_string(),
        Some(ref field) if field == "xFilesFactor" => meta.x_files_factor.to_string(),
        Some(ref field) if field == "aggregationMethod" => meta.aggregation_method.to_string(),
        Some(ref field) if field == "fileSize" => meta.file_size().to_string(),
        Some(ref field) => return Err(format_err!("Unknown field \"{}\". Valid fields are maxRetention, xFilesFactor, aggregationMethod, archives, fileSize", field)),
        None => format!("{:#?}", meta),
    };

    println!("{}", info);

    Ok(())
}
