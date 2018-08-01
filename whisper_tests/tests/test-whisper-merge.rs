extern crate rand;
extern crate whisper;
extern crate whisper_tests;

use std::path::PathBuf;
use whisper::point::*;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

fn create_and_update(path: &PathBuf, timestamps: &[u32], now: u32) -> WhisperFile {
    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .build(path)
        .unwrap();

    for timestamp in timestamps {
        file.update(rand::random(), *timestamp, now).unwrap();
    }
    file
}

fn create_and_update_many(path: &PathBuf, timestamps: &[u32], now: u32) -> WhisperFile {
    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .build(path)
        .unwrap();

    let points: Vec<Point> = timestamps
        .iter()
        .map(|interval| Point {
            interval: *interval,
            value: rand::random(),
        })
        .collect();

    file.update_many(&points, now).unwrap();

    file
}

fn create_and_update_points(path: &PathBuf, points: &[Point], now: u32) -> WhisperFile {
    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .build(path)
        .unwrap();

    file.update_many(&points, now).unwrap();

    file
}

#[test]
fn test_merge_update() {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue34_1");
    let path2 = get_file_path(&temp_dir, "issue34_2");

    let now = 1528240800;

    let mut _file1 = create_and_update(&path1, &[now - 60, now - 180, now - 300], now);
    let mut file2 = create_and_update(&path2, &[now - 120, now - 360, now - 480], now);

    whisper::merge::merge(&path1, &path2, 0, now, now).unwrap();
    let points = file2.dump(60).unwrap();

    for delta in &[60, 180, 300] {
        assert!(
            points.iter().any(|p| p.interval == now - delta),
            "should contain (now - {}) from file1",
            delta
        );
    }

    for delta in &[120, 360, 480] {
        assert!(
            points.iter().any(|p| p.interval == now - delta),
            "should contain (now - {}) from file2",
            delta
        );
    }
}

#[test]
fn test_merge_update_many() {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue34_3");
    let path2 = get_file_path(&temp_dir, "issue34_4");

    let now = 1528240800;

    let mut _file1 = create_and_update_many(&path1, &[now - 60, now - 180, now - 300], now);
    let mut file2 = create_and_update_many(&path2, &[now - 120, now - 360, now - 480], now);

    whisper::merge::merge(&path1, &path2, 0, now, now).unwrap();
    let points = file2.dump(60).unwrap();

    for delta in &[60, 180, 300] {
        assert!(
            points.iter().any(|p| p.interval == now - delta),
            "should contain (now - {}) from file1",
            delta
        );
    }

    for delta in &[120, 360, 480] {
        assert!(
            points.iter().any(|p| p.interval == now - delta),
            "should contain (now - {}) from file2",
            delta
        );
    }
}

#[test]
fn test_merge_overwrite() {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue54_1");
    let path2 = get_file_path(&temp_dir, "issue54_2");

    let now = 1528240800;

    let mut _file1 = create_and_update_points(
        &path1,
        &[
            Point {
                interval: now - 60,
                value: 60.0,
            },
            Point {
                interval: now - 180,
                value: 180.0,
            },
            Point {
                interval: now - 300,
                value: 300.0,
            },
        ],
        now,
    );

    let mut file2 = create_and_update_points(
        &path2,
        &[
            Point {
                interval: now - 120,
                value: 120.0,
            },
            Point {
                interval: now - 300,
                value: 3000.0,
            },
            Point {
                interval: now - 360,
                value: 360.0,
            },
            Point {
                interval: now - 480,
                value: 480.0,
            },
        ],
        now,
    );

    whisper::merge::merge(&path1, &path2, 0, now, now).unwrap();
    let points = file2.dump(60).unwrap();

    for delta in &[60, 180, 300] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta)),
            "should contain (now - {} = {}, {}) from file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    for delta in &[120, 360, 480] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta)),
            "should contain (now - {} = {}, {}) from file2, points: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }
}

#[test]
fn test_fill_overlap() {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue54_1");
    let path2 = get_file_path(&temp_dir, "issue54_2");

    let now = 1528240800;

    let mut _file1 = create_and_update_points(
        &path1,
        &[
            Point {
                interval: now - 60,
                value: 60.0,
            },
            Point {
                interval: now - 180,
                value: 180.0,
            },
            Point {
                interval: now - 300,
                value: 3000.0,
            },
        ],
        now,
    );

    let mut file2 = create_and_update_points(
        &path2,
        &[
            Point {
                interval: now - 120,
                value: 120.0,
            },
            Point {
                interval: now - 300,
                value: 300.0,
            },
            Point {
                interval: now - 360,
                value: 360.0,
            },
            Point {
                interval: now - 480,
                value: 480.0,
            },
        ],
        now,
    );

    whisper::fill::fill(&path1, &path2, now, now).unwrap();
    let points = file2.dump(60).unwrap();

    for delta in &[180] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta)),
            "should contain (now - {} = {}, {}) from file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    for delta in &[120, 300, 360, 480] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta)),
            "should contain (now - {} = {}, {}) from file2, points: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }
}
