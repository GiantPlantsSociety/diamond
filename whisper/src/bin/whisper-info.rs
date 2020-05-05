use serde_json::json;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::exit;
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

fn format_info(meta: &whisper::WhisperMetadata, json: bool) -> Result<(), Box<dyn Error>> {
    if json {
        let john = json!({
            "maxRetention": &meta.max_retention,
            "xFilesFactor": &meta.x_files_factor,
            "aggregationMethod": &meta.aggregation_method.to_string(),
            "fileSize": &meta.file_size(),
            "archives": &meta.archives
                .iter()
                .map(|a| json!({
                    "retention": a.retention(),
                    "secondsPerPoint": a.seconds_per_point,
                    "points": a.points,
                    "size": a.size(),
                    "offset": a.offset,
                    }))
                .collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&john)?);
    } else {
        println!("maxRetention: {}", &meta.max_retention);
        println!("xFilesFactor: {}", &meta.x_files_factor);
        println!("aggregationMethod: {}", &meta.aggregation_method);
        println!("fileSize: {}", &meta.file_size());
        println!();

        for (i, archive) in meta.archives.iter().enumerate() {
            println!("Archive {}", i);
            println!("retention: {}", &archive.retention());
            println!("secondsPerPoint: {}", &archive.seconds_per_point);
            println!("points: {}", &archive.points);
            println!("size: {}", &archive.size());
            println!("offset: {}", &archive.offset);
            println!();
        }
    }

    Ok(())
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let file = whisper::WhisperFile::open(&args.path)?;
    let meta = file.info();

    match &args.field {
        Some(ref field) if field == "maxRetention"      => println!("{}", meta.max_retention),
        Some(ref field) if field == "xFilesFactor"      => println!("{}", meta.x_files_factor),
        Some(ref field) if field == "aggregationMethod" => println!("{}", meta.aggregation_method),
        Some(ref field) if field == "fileSize"          => println!("{}", meta.file_size()),
        Some(ref field) => return Err(
            Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown field \"{}\". Valid fields are maxRetention, xFilesFactor, aggregationMethod, archives, fileSize", field)
        ))),
        None => format_info(&meta, args.json)?,
    };

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
