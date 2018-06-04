#[macro_use]
extern crate bencher;
extern crate whisper;
extern crate whisper_tests;
extern crate rand;

use bencher::Bencher;
use rand::*;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::WhisperMetadata;
use whisper::retention::Retention;
use whisper::aggregation::AggregationMethod;
use whisper_tests::*;
use whisper::interval::Interval;


fn create_metadata() -> WhisperMetadata {
    let retentions = vec![
        Retention { seconds_per_point: 1, points: 300 },
        Retention { seconds_per_point: 60, points: 30 },
        Retention { seconds_per_point: 300, points: 12 },
    ];
    WhisperMetadata::create(&retentions, 0.1, AggregationMethod::Average).expect("Retentions")
}

fn current_time() -> u32 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time");
    let millis: u32 = since_epoch.as_secs() as u32 * 1000 + since_epoch.subsec_nanos() / 1_000_000;
    millis
}

fn test_create(bench: &mut Bencher) {
    let mut cleaner: CleanTempDir = CleanTempDir::new();
    let meta = create_metadata();
    let mut index = 1;
    let i = &mut index;
    bench.iter(|| {
        let path = cleaner.get_file_path("whisper_create", "wsp");

        whisper::create(&meta, &path, false).expect("creating");
        *i += 1;
    });
}

fn test_update(bench: &mut Bencher) {
    let mut cleaner: CleanTempDir = CleanTempDir::new();
    let meta = create_metadata();
    let path = cleaner.get_file_path("whisper_update", "wsp");
    whisper::create(&meta, &path, false).expect("Create file for update");

    let seconds_ago = 3500;
    let mut current_value = 0.5;
    let i = &mut current_value;
    let step = 0.2;
    let now = current_time();

    bench.iter(|| {
        for j in 0..seconds_ago {
            whisper::update(&path, *i, now - seconds_ago + j, now).expect("update");
            *i += step;
        }
    });
}

fn test_fetch(bench: &mut Bencher) {
    let mut cleaner: CleanTempDir = CleanTempDir::new();
    let meta = create_metadata();
    let path = cleaner.get_file_path("whisper_fetch", "wsp");
    whisper::create(&meta, &path, false).expect("Create file for fetching");

    let seconds_ago = 3500;
    let mut current_value = 0.5;
    let i = &mut current_value;
    let step = 0.2;
    let now = current_time();

    for j in 0..seconds_ago {
        whisper::update(&path, *i, now - seconds_ago + j, now).expect("update");
        *i += step;
    };

    let from_time = now - seconds_ago;
    let until_time = from_time + 1000;
    let interval = Interval::new(from_time, until_time).expect("interval");
    bench.iter(|| {
        // TODO: seconds per point is not in python api
        whisper::fetch(&path, interval, now, rand::thread_rng().next_u32());
    });
}



benchmark_group!(
benches,
//test_create,
//test_update,
test_fetch,
);
benchmark_main!(benches);