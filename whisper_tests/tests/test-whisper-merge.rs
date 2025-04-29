use std::error::Error;
use whisper::builder;
use whisper::builder::*;
use whisper::point::*;
use whisper::retention::*;
use whisper_tests::*;

#[test]
#[allow(clippy::unreadable_literal)]
fn test_merge_update() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue34_1");
    let path2 = get_file_path(&temp_dir, "issue34_2");

    let now = 1528240800;

    let mut _file1 = create_and_update_many(&path1, &[now - 60, now - 180, now - 300], now)?;
    let mut file2 = create_and_update_many(&path2, &[now - 120, now - 360, now - 480], now)?;

    whisper::merge::merge(&path1, &path2, 0, now, now)?;
    let points = file2.dump(60)?;

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

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_merge_update_many() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue34_3");
    let path2 = get_file_path(&temp_dir, "issue34_4");

    let now = 1528240800;

    let mut _file1 = create_and_update_many(&path1, &[now - 60, now - 180, now - 300], now)?;
    let mut file2 = create_and_update_many(&path2, &[now - 120, now - 360, now - 480], now)?;

    whisper::merge::merge(&path1, &path2, 0, now, now)?;
    let points = file2.dump(60)?;

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

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_merge_errors() -> Result<(), builder::BuilderError> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "issue34_7");
    let path2 = get_file_path(&temp_dir, "issue34_8");
    let path3 = get_file_path(&temp_dir, "issue34_9");

    let now = 1528240800;

    let _file1 = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 11,
        })
        .build(&path1)?;

    let _file2 = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 12,
        })
        .build(&path2)?;

    let _file3 = WhisperBuilder::default()
        .add_retention(Retention {
            seconds_per_point: 60,
            points: 11,
        })
        .build(&path3)?;

    assert!(whisper::merge::merge(&path1, &path2, 0, now, now).is_err());
    assert!(whisper::merge::merge(&path1, &path3, now - 10, now - 20, now).is_err());

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_merge_overwrite() -> Result<(), Box<dyn Error>> {
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
    )?;

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
    )?;

    whisper::merge::merge(&path1, &path2, 0, now, now)?;
    let points = file2.dump(60)?;

    for delta in &[60, 180, 300] {
        assert!(
            points.iter().any(|p| p.interval == (now - delta)
                && (p.value - f64::from(*delta)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    for delta in &[120, 360, 480] {
        assert!(
            points.iter().any(|p| p.interval == (now - delta)
                && (p.value - f64::from(*delta)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from file2, points: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_fill_overlap() -> Result<(), Box<dyn Error>> {
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
    )?;

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
    )?;

    whisper::fill::fill(&path1, &path2, now, now)?;
    let points = file2.dump(60)?;

    {
        let delta = &180;
        assert!(
            points.iter().any(|p| p.interval == (now - delta)
                && (p.value - f64::from(*delta)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    for delta in &[120, 300, 360, 480] {
        assert!(
            points.iter().any(|p| p.interval == (now - delta)
                && (p.value - f64::from(*delta)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from file2, points: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    Ok(())
}
