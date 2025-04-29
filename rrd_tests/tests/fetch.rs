use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use tempfile::{Builder, TempDir};

fn get_temp_dir() -> TempDir {
    Builder::new()
        .prefix("rrd")
        .tempdir()
        .expect("Temp dir created")
}

fn get_file_path(temp_dir: &TempDir, prefix: &str) -> PathBuf {
    let file_name = format!("{}_{}.rrd", prefix, random_string(10));
    let mut path = temp_dir.path().to_path_buf();
    path.push(file_name);
    path
}

fn random_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rng(), len)
}

#[test]
fn test_read_rrd() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "read_rrd");

    Command::new("rrdtool")
        .arg("create")
        .arg(&path)
        .args(["--step", "300"])
        .arg("DS:temp:GAUGE:600:-273:5000")
        .arg("RRA:AVERAGE:0.5:1:1200")
        .arg("RRA:MIN:0.6:12:2400")
        .arg("RRA:MAX:0.7:12:2400")
        .arg("RRA:AVERAGE:0.8:12:2400")
        .status()?;

    let rrd_info = rrd::info(&path, None, false)?;

    let info: HashMap<String, rrd::Value> = rrd_info.iter().collect();
    let seconds_per_pdp = info["step"].as_long().unwrap();
    assert_eq!(seconds_per_pdp, 300);

    let mut datasources = BTreeSet::new();
    datasources.insert("temp".to_owned());
    assert_eq!(rrd_info.datasources(), datasources);

    assert_eq!(rrd_info.rra_count(), 4);

    let rras = rrd_info.rras();
    assert_eq!(rras[0].cf, rrd::AggregationMethod::Average);
    assert_eq!(rras[0].rows, 1200);
    assert_eq!(rras[0].pdp_per_row, 1);
    assert_eq!(rras[0].xff, 0.5);

    assert_eq!(rras[1].cf, rrd::AggregationMethod::Min);
    assert_eq!(rras[1].rows, 2400);
    assert_eq!(rras[1].pdp_per_row, 12);
    assert_eq!(rras[1].xff, 0.6);

    assert_eq!(rras[2].cf, rrd::AggregationMethod::Max);
    assert_eq!(rras[2].rows, 2400);
    assert_eq!(rras[2].pdp_per_row, 12);
    assert_eq!(rras[2].xff, 0.7);

    assert_eq!(rras[3].cf, rrd::AggregationMethod::Average);
    assert_eq!(rras[3].rows, 2400);
    assert_eq!(rras[3].pdp_per_row, 12);
    assert_eq!(rras[3].xff, 0.8);

    Ok(())
}
