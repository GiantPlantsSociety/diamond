use std::error::Error;

use whisper::aggregation::*;
use whisper::point::*;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

#[tokio::test]
#[allow(clippy::unreadable_literal)]
async fn test_resize_simple_long() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "resize_1");
    let path2 = get_file_path(&temp_dir, "resize_2");

    let now = 1528240800;

    let original_points = &(1..10)
        .map(|x| Point {
            interval: now - 60 * x,
            value: 60.0 * f64::from(x),
        })
        .collect::<Vec<Point>>();

    let mut _file1 = create_and_update_points(&path1, original_points, now);

    let retentions = &[Retention {
        seconds_per_point: 60,
        points: 20,
    }];

    whisper::resize::resize(
        &path1,
        Some(&path2),
        retentions,
        0.5,
        AggregationMethod::Average,
        false,
        true,
        now,
    )
    .await?;

    let mut file2 = WhisperFile::open(&path2).await?;

    let points = file2.dump(60).await?;

    for delta in 1..10 {
        assert!(
            points.iter().any(|p| p.interval == (now - delta * 60)
                && (p.value - f64::from(delta * 60)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from original file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    assert_eq!(points.len(), 20, "Should be 20 points");

    let zero_count = points.iter().filter(|x| *x == &Point::default()).count();
    assert_eq!(zero_count, 11, "Should be 11 empty points {:?}", &points);

    Ok(())
}

#[tokio::test]
async fn test_resize_simple_short() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "resize_3");
    let path2 = get_file_path(&temp_dir, "resize_4");

    let now = 1528240800;

    let original_points = &(1..10)
        .map(|x| Point {
            interval: now - 60 * x,
            value: 60.0 * f64::from(x),
        })
        .collect::<Vec<Point>>();

    let mut _file1 = create_and_update_points(&path1, original_points, now);

    let retentions = &[Retention {
        seconds_per_point: 60,
        points: 5,
    }];

    whisper::resize::resize(
        &path1,
        Some(&path2),
        retentions,
        0.5,
        AggregationMethod::Average,
        false,
        true,
        now,
    )
    .await?;

    let mut file2 = WhisperFile::open(&path2).await?;

    let points = file2.dump(60).await?;

    assert_eq!(points.len(), 5, "Should be 5 points");

    for delta in 1..6 {
        assert!(
            points.iter().any(|p| p.interval == (now - delta * 60)
                && (p.value - f64::from(delta * 60)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from original file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_resize_extend_short() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "resize_3");
    let path2 = get_file_path(&temp_dir, "resize_4");

    let now = 1528240800;

    let original_points = &(1..11)
        .map(|x| Point {
            interval: now - 60 * x,
            value: 60.0 * f64::from(x),
        })
        .collect::<Vec<Point>>();

    let mut _file1 = create_and_update_points(&path1, original_points, now);

    let retentions = &[
        Retention {
            seconds_per_point: 60,
            points: 5,
        },
        Retention {
            seconds_per_point: 120,
            points: 5,
        },
    ];

    whisper::resize::resize(
        &path1,
        Some(&path2),
        retentions,
        0.5,
        AggregationMethod::Average,
        false,
        true,
        now,
    )
    .await?;

    let mut file2 = WhisperFile::open(&path2).await?;

    let points = file2.dump(60).await?;

    assert_eq!(points.len(), 5, "Should be 5 points");

    for delta in 1..6 {
        assert!(
            points.iter().any(|p| p.interval == (now - delta * 60)
                && (p.value - f64::from(delta * 60)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from original file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_resize_aggr_simple_long() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "resize_1");
    let path2 = get_file_path(&temp_dir, "resize_2");

    let now = 1528240800;

    let original_points = &(1..10)
        .map(|x| Point {
            interval: now - 60 * x,
            value: 60.0 * f64::from(x),
        })
        .collect::<Vec<Point>>();

    let mut _file1 = create_and_update_points(&path1, original_points, now);

    let retentions = &[Retention {
        seconds_per_point: 60,
        points: 20,
    }];

    whisper::resize::resize(
        &path1,
        Some(&path2),
        retentions,
        0.5,
        AggregationMethod::Average,
        true,
        true,
        now,
    )
    .await?;

    let mut file2 = WhisperFile::open(&path2).await?;

    let points = file2.dump(60).await?;

    for delta in 1..10 {
        assert!(
            points.iter().any(|p| p.interval == (now - delta * 60)
                && (p.value - f64::from(delta * 60)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from original file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    assert_eq!(points.len(), 20, "Should be 20 points");

    let zero_count = points.iter().filter(|x| *x == &Point::default()).count();
    assert_eq!(zero_count, 11, "Should be 11 empty points {:?}", &points);

    Ok(())
}

#[tokio::test]
async fn test_resize_aggr_simple_short() -> Result<(), Box<dyn Error>> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "resize_7");
    let path2 = get_file_path(&temp_dir, "resize_8");

    let now = 1528240800;

    let original_points = &(1..10)
        .map(|x| Point {
            interval: now - 60 * x,
            value: 60.0 * f64::from(x),
        })
        .collect::<Vec<Point>>();

    let mut _file1 = create_and_update_points(&path1, original_points, now);

    let retentions = &[Retention {
        seconds_per_point: 60,
        points: 5,
    }];

    whisper::resize::resize(
        &path1,
        Some(&path2),
        retentions,
        0.5,
        AggregationMethod::Average,
        true,
        true,
        now,
    )
    .await?;

    let mut file2 = WhisperFile::open(&path2).await?;

    let points = file2.dump(60).await?;

    assert_eq!(points.len(), 5, "Should be 5 points");

    for delta in 1..6 {
        assert!(
            points.iter().any(|p| p.interval == (now - delta * 60)
                && (p.value - f64::from(delta * 60)) < std::f64::EPSILON),
            "should contain (now - {} = {}, {}) from original file1: {:?}",
            delta,
            now - delta,
            delta,
            points
        );
    }

    Ok(())
}
