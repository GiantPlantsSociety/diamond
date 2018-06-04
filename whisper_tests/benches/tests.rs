#[macro_use]
extern crate bencher;
extern crate whisper;
extern crate whisper_tests;

use bencher::Bencher;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::WhisperMetadata;
use whisper::retention::Retention;
use whisper::archive_info::ArchiveInfo;
use whisper::aggregation::AggregationMethod;
use whisper_tests::*;


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
    let millis: u32 = since_epoch.as_secs() as u32 * 1000 + since_epoch.subsec_nanos() as u32 / 1_000_000;
    millis
}

fn test_create(bench: &mut Bencher) {
    let cleaner: CleanDir = CleanDir::new();
    let meta = create_metadata();
    let mut index = 1;
    let i = &mut index;
    bench.iter(|| {
        let path = PathBuf::from(format!("{}/whisper_{}.wsp", BENCH_FILE_PATH, i));

        whisper::create(&meta, &path, false).expect("creating");
        *i += 1;
    });
}

fn test_update(bench: &mut Bencher) {
    let clean: CleanDir = CleanDir::new();
    let meta = create_metadata();
    let path = PathBuf::from(format!("{}/whisper_update_test.wsp", BENCH_FILE_PATH));
    whisper::create(&meta, &path, false).expect("Create file for update");

    let seconds_ago = 3500;
    let mut current_value = 0.5;
    let i = &mut current_value;
    let mut index = 0;
    let k = &mut index;
    let step = 0.2;
    let now = current_time();

    bench.iter(|| {
        whisper::update(&path, *i, now - seconds_ago + *k, now as u32).expect("update");
        *k += 1;
        *i += step;
    });
}

benchmark_group!(
benches,
test_create,
//test_update
);
benchmark_main!(benches);