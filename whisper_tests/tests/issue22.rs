extern crate whisper;
extern crate whisper_tests;

use whisper::point::*;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

#[test]
fn issue22() {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 1, points: 10 })
        .build(path)
        .unwrap();

    let now = 1000;

    file.update(&Point { interval: now - 1, value: 100.0 }, now).unwrap();
    file.update(&Point { interval: now - 2, value: 200.0 }, now).unwrap();

    let points = file.dump(1).unwrap();

    assert_eq!(points[0].interval, now - 1);
    assert_eq!(points[9].interval, now - 2);
}
