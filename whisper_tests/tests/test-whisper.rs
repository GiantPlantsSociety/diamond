use failure::*;

use whisper::builder::WhisperBuilder;
use whisper::interval::Interval;
use whisper::point::Point;
use whisper::retention::Retention;
use whisper::suggest_archive;
use whisper_tests::{get_temp_dir, get_file_path};

#[test]
fn whisper_suggest_archive() -> Result<(), Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "suggest");

    let now = 1528240800;

    let mut file = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 10,
        })
        .add_retention(Retention {
            seconds_per_point: 300,
            points: 10,
        })
        .add_retention(Retention {
            seconds_per_point: 600,
            points: 10,
        })
        .build(path)?;

    let points: Vec<Point> = [
        now - 60,
        now - 300,
        now - 360,
        now - 600,
        now - 900,
        now - 1200,
        now - 3600,
        now - 6000,
    ]
    .iter()
    .map(|interval| Point {
        interval: *interval,
        value: rand::random(),
    })
    .collect();

    file.update_many(&points, now)?;

    assert_eq!(
        suggest_archive(
            &file,
            Interval::new(now - 600, now - 60).map_err(err_msg)?,
            now
        ),
        Some(60)
    );

    assert_eq!(
        suggest_archive(
            &file,
            Interval::new(now - 1200, now - 600).map_err(err_msg)?,
            now
        ),
        Some(300)
    );

    assert_eq!(
        suggest_archive(
            &file,
            Interval::new(now - 6000, now - 3000).map_err(err_msg)?,
            now
        ),
        Some(600)
    );

    assert_eq!(
        suggest_archive(
            &file,
            Interval::new(now - 12000, now - 6060).map_err(err_msg)?,
            now
        ),
        None
    );

    Ok(())
}
