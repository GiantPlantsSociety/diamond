use bencher::Bencher;
use bencher::{benchmark_main, benchmark_group};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::WhisperFile;
use whisper::builder::{WhisperBuilder, BuilderError};
use whisper::retention::Retention;
use whisper::point::Point;
use whisper::interval::Interval;
use whisper_tests::*;

const SECONDS_AGO: u32 = 3500;
const VALUE_STEP: f64 = 0.2;


fn create_file(path: &Path) -> Result<WhisperFile, BuilderError> {
    WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 1, points: 300 })
        .add_retention(Retention { seconds_per_point: 60, points: 30 })
        .add_retention(Retention { seconds_per_point: 300, points: 12 })
        .x_files_factor(0.1)
        .build(path)
}

fn current_time() -> u32 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time");
    since_epoch.as_secs() as u32
}

fn test_create(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let mut index = 1;
    let i = &mut index;
    bench.iter(|| {
        let path = get_file_path(&temp_dir, "whisper_create");
        create_file(&path).expect("creating");
        *i += 1;
    });
}

fn test_update(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "whisper_update");
    let mut file = create_file(&path).expect("Create file for update");

    let mut current_value = 0.5;
    let i = &mut current_value;
    let now = current_time();

    bench.iter(|| {
        for j in 0..SECONDS_AGO {
            file.update(&Point { interval: now - SECONDS_AGO + j, value: *i}, now).expect("update");
            *i += VALUE_STEP;
        }
    });
}

fn test_fetch(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "whisper_fetch");
    let mut file = create_file(&path).expect("Create file for fetching");

    let mut current_value = 0.5;
    let now = current_time();

    for j in 0..SECONDS_AGO {
        file.update(&Point { interval: now - SECONDS_AGO + j, value: current_value}, now).expect("update");
        current_value += VALUE_STEP;
    };

    let from_time = now - SECONDS_AGO;
    let until_time = from_time + 1000;
    let interval = Interval::new(from_time, until_time).expect("interval");
    bench.iter(|| {
        let seconds_per_point = file.suggest_archive(interval, now).expect("Archive");
        file.fetch(seconds_per_point, interval, now).expect("fetch");
    });
}

fn test_update_fetch(bench: &mut Bencher) {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "whisper_update");
    let mut file = create_file(&path).expect("Create file for update");

    let mut current_value = 0.5;
    let i = &mut current_value;
    let now = current_time();

    let from_time = now - SECONDS_AGO;
    let until_time = from_time + 1000;
    let interval = Interval::new(from_time, until_time).expect("interval");
    bench.iter(|| {
        for j in 0..SECONDS_AGO {
            file.update(&Point { interval: now - SECONDS_AGO + j, value: *i}, now).expect("update");
            *i += VALUE_STEP;
        }
        let seconds_per_point = file.suggest_archive(interval, now).expect("Archive");
        file.fetch(seconds_per_point, interval, now).expect("fetch");
    });
}

benchmark_group!(benches,
    test_create,
    test_update,
    test_fetch,
    test_update_fetch,
);
benchmark_main!(benches);
