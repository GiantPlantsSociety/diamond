use whisper::point::*;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

#[test]
fn issue22_original() -> Result<(), failure::Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 1, points: 10 })
        .build(path)?;

    let now = 1000;

    file.update(&Point { interval: now - 1, value: 100.0 }, now)?;
    file.update(&Point { interval: now - 2, value: 200.0 }, now)?;

    let points = file.dump(1)?;

    assert_eq!(points[0].interval, now - 1);
    assert_eq!(points[9].interval, now - 2);

    Ok(())
}

#[test]
fn issue22_many_archives() -> Result<(), failure::Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22_many");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 2, points: 10 })
        .add_retention(Retention { seconds_per_point: 4, points: 10 })
        .build(path)?;

    let now = 1000;

    for item in (2..18).step_by(2) {
        let delta = 18 - item;
        file.update(&Point {
                interval: now - delta,
                value: f64::from(delta) * 100.0,
            },
            now,
        )?
    }

    let points = file.dump(2)?;

    for delta in &[2, 4, 6, 8, 10, 12, 14, 16] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            delta * 100,
            points
        );
    }

    let points2 = file.dump(4)?;

    for delta in &[4, 8, 12, 16] {
        assert!(
            points2
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == (f64::from(*delta - 1)) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            (delta - 1) * 100,
            points2
        );
    }

    Ok(())
}

#[test]
fn issue22_many_archives_once() -> Result<(), failure::Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22_many");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 2, points: 10 })
        .add_retention(Retention { seconds_per_point: 4, points: 10 })
        .build(path)?;

    let now = 1000;

    file.update_many(
        &[
            Point { interval: now - 16, value: 1600.0 },
            Point { interval: now - 14, value: 1400.0 },
            Point { interval: now - 12, value: 1200.0 },
            Point { interval: now - 10, value: 1000.0 },
            Point { interval: now - 8, value: 800.0 },
            Point { interval: now - 6, value: 600.0 },
            Point { interval: now - 4, value: 400.0 },
            Point { interval: now - 2, value: 200.0 },
        ],
        now,
    )?;

    let points = file.dump(2)?;

    for delta in &[2, 4, 6, 8, 10, 12, 14, 16] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == (f64::from(*delta)) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            delta * 100,
            points
        );
    }

    let points2 = file.dump(4)?;

    for delta in &[4, 8, 12, 16] {
        assert!(
            points2
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == (f64::from(*delta - 1)) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            (delta - 1) * 100,
            points2
        );
    }

    Ok(())
}

#[test]
fn issue22_many_archives_reverse() -> Result<(), failure::Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22_many");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 2, points: 10 })
        .add_retention(Retention { seconds_per_point: 4, points: 10 })
        .build(path)?;

    let now = 1000;

    for delta in (2..18).step_by(2) {
        file.update(
            &Point {
                interval: now - delta,
                value: f64::from(delta) * 100.0,
            },
            now,
        )?
    }

    let points = file.dump(2)?;

    for delta in &[2, 4, 6, 8, 10, 12, 14, 16] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            delta * 100,
            points
        );
    }

    let points2 = file.dump(4)?;

    for delta in &[4, 8, 12, 16] {
        assert!(
            points2
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == (f64::from(*delta - 1)) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            (delta - 1) * 100,
            points2
        );
    }

    Ok(())
}

#[test]
fn issue22_many_archives_once_shuffle() -> Result<(), failure::Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "issue22_many");

    let mut file = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 2, points: 10 })
        .add_retention(Retention { seconds_per_point: 4, points: 10 })
        .build(path)?;

    let now = 1000;

    for point in &[
        Point { interval: now - 16, value: 1600.0 },
        Point { interval: now - 2, value: 200.0 },
        Point { interval: now - 12, value: 1200.0 },
        Point { interval: now - 6, value: 600.0 },
        Point { interval: now - 10, value: 1000.0 },
        Point { interval: now - 8, value: 800.0 },
        Point { interval: now - 4, value: 400.0 },
        Point { interval: now - 14, value: 1400.0 },
    ] {
        file.update(point, now)?
    }

    let points = file.dump(2)?;

    for delta in &[2, 4, 6, 8, 10, 12, 14, 16] {
        assert!(
            points
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == f64::from(*delta) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            delta * 100,
            points
        );
    }

    let points2 = file.dump(4)?;

    for delta in &[4, 8, 12, 16] {
        assert!(
            points2
                .iter()
                .any(|p| p.interval == (now - delta) && p.value == (f64::from(*delta - 1)) * 100.0),
            "should contain (now - {} = {}, {}), points: {:?}",
            delta,
            now - delta,
            (delta - 1) * 100,
            points2
        );
    }

    Ok(())
}
