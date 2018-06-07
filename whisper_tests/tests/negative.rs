extern crate whisper;
extern crate whisper_tests;

use whisper::*;
use whisper::aggregation::AggregationMethod;
use whisper::retention::*;
use whisper::point::Point;
use whisper_tests::*;

#[test]
fn negative_write_issue() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "negative_issue");

    let meta = WhisperMetadata::create(
        &[Retention { seconds_per_point: 60, points: 10 }],
        0.5,
        AggregationMethod::Average
    ).unwrap();

    println!("Meta {:?}", &meta);
    create(&meta, &path, false).unwrap();

    update_many(&path, &[
        Point { interval: 1528240818, value: 123.0 },
        Point { interval: 1528240878, value: 124.0 },
        Point { interval: 1528240938, value: 125.0 },
        Point { interval: 1528240998, value: 126.0 },
        Point { interval: 1528241058, value: 127.0 },
        Point { interval: 1528241118, value: 128.0 },
    ], 1528241179).unwrap();

    let points = read_archive_all(&path, &meta.archives[0]).unwrap();
    println!("1: {:?}", points);

    update_many(&path, &[
        Point { interval: 1528240578, value: 223.0 },
        Point { interval: 1528240638, value: 224.0 },
        Point { interval: 1528240698, value: 225.0 },
        Point { interval: 1528240758, value: 226.0 },
    ], 1528241181).unwrap();


    let points = read_archive_all(&path, &meta.archives[0]).unwrap();
    println!("2: {:?}", points);
}
