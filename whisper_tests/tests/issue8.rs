extern crate whisper;
extern crate whisper_tests;

use whisper::*;
use whisper::aggregation::AggregationMethod;
use whisper::retention::*;
use whisper::point::Point;
use whisper_tests::*;

#[test]
fn issue8_many() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue8");

    let meta = WhisperMetadata::create(
        &[Retention { seconds_per_point: 60, points: 10 }],
        0.5,
        AggregationMethod::Average
    ).unwrap();

    create(&meta, &path, false).unwrap();

    update_many(&path, &[Point { interval: 1528240818, value: 123.0 }], 1528240900).unwrap();

    let points = read_archive_all(&path, &meta.archives[0]).unwrap();
    assert_eq!(points[0].interval, 1528240800);
}

#[test]
fn issue8_single() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue8");

    let meta = WhisperMetadata::create(
        &[Retention { seconds_per_point: 60, points: 10 }],
        0.5,
        AggregationMethod::Average
    ).unwrap();

    create(&meta, &path, false).unwrap();

    update(&path, 123.0, 1528240818, 1528240900).unwrap();

    let points = read_archive_all(&path, &meta.archives[0]).unwrap();
    assert_eq!(points[0].interval, 1528240800);
}
