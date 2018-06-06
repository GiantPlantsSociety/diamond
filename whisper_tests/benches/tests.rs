#[macro_use]
extern crate bencher;
extern crate whisper;
extern crate whisper_tests;

use bencher::Bencher;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::WhisperMetadata;
use whisper::retention::Retention;
use whisper::suggest_archive;
use whisper::aggregation::AggregationMethod;
use whisper::interval::Interval;
use whisper_tests::*;

const SECONDS_AGO: u32 = 3500;
const VALUE_STEP: f64 = 0.2;


fn create_metadata() -> WhisperMetadata {
    let retentions = vec![
        Retention { seconds_per_point: 1, points: 300 },
        Retention { seconds_per_point: 60, points: 30 },
        Retention { seconds_per_point: 300, points: 12 },
    ];
    WhisperMetadata::create(&retentions, 0.1, AggregationMethod::Average).expect("Metadadata")
}

fn current_time() -> u32 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time");
    since_epoch.as_secs() as u32
}

fn test_create(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let meta = create_metadata();
    let mut index = 1;
    let i = &mut index;
    bench.iter(|| {
        let path = get_file_path(&temp_dir, "whisper_create");
        whisper::create(&meta, &path, false).expect("creating");
        *i += 1;
    });
}

fn test_update(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let meta = create_metadata();
    let path = get_file_path(&temp_dir, "whisper_update");
    whisper::create(&meta, &path, false).expect("Create file for update");

    let mut current_value = 0.5;
    let i = &mut current_value;
    let now = current_time();

    bench.iter(|| {
        for j in 0..SECONDS_AGO {
            whisper::update(&path, *i, now - SECONDS_AGO + j, now).expect("update");
            *i += VALUE_STEP;
        }
    });
}

fn test_fetch(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let meta = create_metadata();
    let path = get_file_path(&temp_dir, "whisper_fetch");
    whisper::create(&meta, &path, false).expect("Create file for fetching");

    let mut current_value = 0.5;
    let now = current_time();

    for j in 0..SECONDS_AGO {
        whisper::update(&path, current_value, now - SECONDS_AGO + j, now).expect("update");
        current_value += VALUE_STEP;
    };

    let from_time = now - SECONDS_AGO;
    let until_time = from_time + 1000;
    let interval = Interval::new(from_time, until_time).expect("interval");
    bench.iter(|| {
        let archive = suggest_archive(&meta, interval, now).expect("Archive");
        whisper::fetch(&path, interval, now, archive.seconds_per_point).expect("fetch");
    });
}

fn test_update_fetch(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let meta = create_metadata();
    let path = get_file_path(&temp_dir, "whisper_update");
    whisper::create(&meta, &path, false).expect("Create file for update");

    let mut current_value = 0.5;
    let i = &mut current_value;
    let now = current_time();

    let from_time = now - SECONDS_AGO;
    let until_time = from_time + 1000;
    let interval = Interval::new(from_time, until_time).expect("interval");
    bench.iter(|| {
        for j in 0..SECONDS_AGO {
            whisper::update(&path, *i, now - SECONDS_AGO + j, now).expect("update");
            *i += VALUE_STEP;
        }
        let archive = suggest_archive(&meta, interval, now).expect("Archive");
        whisper::fetch(&path, interval, now, archive.seconds_per_point).expect("fetch");
    });
}

benchmark_group!(benches,
    test_create,
    test_update,
    test_fetch,
    test_update_fetch,
);
benchmark_main!(benches);
