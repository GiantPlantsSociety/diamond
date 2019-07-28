use failure::*;

use whisper::builder::WhisperBuilder;
use whisper::interval::Interval;
use whisper::point::Point;
use whisper::retention::Retention;
use whisper::ArchiveData;
use whisper_tests::*;

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
        file.suggest_archive(Interval::new(now - 600, now - 60).map_err(err_msg)?, now),
        Some(60)
    );

    assert_eq!(
        file.suggest_archive(Interval::new(now - 1200, now - 600).map_err(err_msg)?, now),
        Some(300)
    );

    assert_eq!(
        file.suggest_archive(Interval::new(now - 6000, now - 3000).map_err(err_msg)?, now),
        Some(600)
    );

    assert_eq!(
        file.suggest_archive(
            Interval::new(now - 12000, now - 6060).map_err(err_msg)?,
            now
        ),
        None
    );

    Ok(())
}

#[test]
fn whisper_fetch_auto_points() -> Result<(), Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "fetch_auto_points");

    let now = 1528240800;

    let mut file = create_and_update_points(
        &path,
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
            Point {
                interval: now - 540,
                value: 540.0,
            },
        ],
        now,
    )?;

    let interval = Interval::new(now - 540, now - 60).map_err(err_msg)?;

    let data = ArchiveData {
        from_interval: 1528240260,
        until_interval: 1528240740,
        step: 60,
        values: vec![
            Some(540.0),
            None,
            None,
            None,
            Some(300.0),
            None,
            Some(180.0),
            None,
        ],
    };

    assert_eq!(file.fetch_auto_points(interval, now)?, data);

    let interval2 = Interval::new(now - 3000, now - 900).map_err(err_msg)?;
    assert!(file.fetch_auto_points(interval2, now).is_err());

    Ok(())
}
