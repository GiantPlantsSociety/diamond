use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
/// https://oss.oetiker.ch/rrdtool/doc/rrdcreate.en.html
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::{WhisperBuilder, point::Point, retention::Retention};

// # Ignore SIGPIPE
// signal.signal(signal.SIGPIPE, signal.SIG_DFL)

#[derive(Debug, clap::Parser)]
struct Args {
    /// The xFilesFactor to use in the output file. Defaults to the input RRD's xFilesFactor.
    #[arg(long = "xFilesFactor")]
    x_files_factor: Option<f64>,

    /// The consolidation function to fetch from on input and aggregationMethod to set on output. One of: average, last, max, min, avg_zero.
    #[arg(long = "aggregationMethod", default_value = "average")]
    aggregation_method: rrd::AggregationMethod,

    /// Path to place created whisper file. Defaults to the RRD file's source path.
    #[arg(long = "destinationPath")]
    destination_path: Option<PathBuf>,

    rrd_path: PathBuf,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let rrd_info = rrd::info(&args.rrd_path, None, false).unwrap();

    let info: HashMap<String, rrd::Value> = rrd_info.iter().collect();

    let seconds_per_pdp = &info["step"].as_long().unwrap();

    let rras = rrd_info.rras();

    let datasources = rrd_info.datasources();

    // Grab the archive configuration
    let relevant_rras: Vec<_> = rras
        .iter()
        .filter(|rra| rra.cf == args.aggregation_method)
        .collect();

    if relevant_rras.is_empty() {
        let method: &str = args.aggregation_method.into();
        return Err(format!(
            "[ERROR] Unable to find any RRAs with consolidation function: {}",
            method
        )
        .into());
    }

    let archives: Vec<_> = relevant_rras
        .iter()
        .map(|rra| Retention {
            seconds_per_point: (rra.pdp_per_row * seconds_per_pdp) as u32,
            points: rra.rows as u32,
        })
        .collect();

    let x_files_factor: f64 = args
        .x_files_factor
        .unwrap_or_else(|| relevant_rras.last().unwrap().xff);

    for datasource in &datasources {
        let suffix = if datasources.len() > 1 {
            format!("_{}", datasource)
        } else {
            String::new()
        };

        let destination_directory = args
            .destination_path
            .as_ref()
            .unwrap_or_else(|| &args.rrd_path);
        let destination_name = format!(
            "{}{}.wsp",
            args.rrd_path.file_stem().unwrap().to_str().unwrap(),
            suffix
        );
        let path = destination_directory.with_file_name(destination_name);

        let mut whisper_file = WhisperBuilder::default()
            .add_retentions(&archives)
            .x_files_factor(x_files_factor as f32)
            // .aggregation_method(args.aggregation_method) // TODO
            .build(&path)?;

        // let size = os.stat(path).st_size;
        // archiveConfig = ",".join(["%d:%d" % ar for ar in archives]);
        // print("Created: %s (%d bytes) with archives: %s" % (path, size, archiveConfig));

        println!("Migrating data");
        let mut archive_number = archives.len();
        for archive in archives.iter().rev() {
            let retention = u64::from(archive.retention());
            let end_time = now - now % u64::from(archive.seconds_per_point);
            let start_time = end_time - retention;
            let data = rrd::fetch(
                &args.rrd_path,
                args.aggregation_method,
                Some(archive.seconds_per_point),
                start_time,
                end_time,
            )?;

            let column_index = data
                .columns
                .iter()
                .position(|column| column == datasource)
                .unwrap();

            let values: Vec<Point> = data
                .rows
                .iter()
                .enumerate()
                .map(|(index, row)| Point {
                    interval: ((data.time_info.start as u64) + (index as u64) * data.time_info.step)
                        as u32,
                    value: row[column_index],
                })
                .collect();

            archive_number -= 1;
            println!(
                " migrating {} datapoints from archive {}",
                values.len(),
                archive_number
            );
            whisper_file.update_many(&values, now as u32)?;
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
