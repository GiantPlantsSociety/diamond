extern crate whisper;
extern crate whisper_tests;

use whisper::point::Point;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

#[test]
fn issue8_many() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue8");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 60, points: 10 })
        .build(path)
        .unwrap();

    file.update_many(&[Point { interval: 1528240818, value: 123.0 }], 1528240900).unwrap();

    let points = file.dump(60).unwrap();
    assert_eq!(points[0].interval, 1528240800);
}

#[test]
fn issue8_single() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue8");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 60, points: 10 })
        .build(path)
        .unwrap();

    file.update(&Point { interval: 1528240818, value: 123.0 }, 1528240900).unwrap();

    let points = file.dump(60).unwrap();
    assert_eq!(points[0].interval, 1528240800);
}
