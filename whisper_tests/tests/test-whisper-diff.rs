use failure::Error;
use whisper::diff::*;
use whisper::point::*;
use whisper::retention::*;
use whisper::*;
use whisper_tests::*;

#[test]
#[allow(clippy::unreadable_literal)]
fn test_diff_simple_filtered() -> Result<(), Error> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "diff1_1");
    let path2 = get_file_path(&temp_dir, "diff1_2");

    let now = 1528240800;

    let _file1 = create_and_update_points(
        &path1,
        &[
            Point { interval: now - 60, value: 60.0 },
            Point { interval: now - 180, value: 180.0 },
            Point { interval: now - 300, value: 300.0 },
        ],
        now,
    )?;

    let _file2 = create_and_update_points(
        &path2,
        &[
            Point { interval: now - 120, value: 120.0 },
            Point { interval: now - 300, value: 3000.0 },
            Point { interval: now - 360, value: 360.0 },
            Point { interval: now - 480, value: 480.0 },
        ],
        now,
    )?;

    let diff_points = whisper::diff::diff(&path1, &path2, true, now, now)?;

    assert_eq!(diff_points[0].points, 1, "there should be 1 point");
    assert_eq!(diff_points[0].total, 1, "there should be total 1");

    assert_eq!(
        diff_points[0].diffs[0],
        DiffPoint {
            interval: now - 300,
            value1: Some(300.0),
            value2: Some(3000.0),
        }
    );

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_diff_simple_unfiltered() -> Result<(), Error> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "diff2_1");
    let path2 = get_file_path(&temp_dir, "diff2_2");

    let now = 1528240800;

    let _file1 = create_and_update_points(
        &path1,
        &[
            Point { interval: now - 60, value: 60.0 },
            Point { interval: now - 180, value: 180.0 },
            Point { interval: now - 300, value: 300.0 },
            Point { interval: now - 540, value: 540.0 },
        ],
        now,
    )?;

    let _file2 = create_and_update_points(
        &path2,
        &[
            Point { interval: now - 120, value: 120.0 },
            Point { interval: now - 300, value: 3000.0 },
            Point { interval: now - 360, value: 360.0 },
            Point { interval: now - 480, value: 480.0 },
            Point { interval: now - 540, value: 540.0 },
        ],
        now,
    )?;

    let diff_points = whisper::diff::diff(&path1, &path2, false, now, now)?;

    assert_eq!(diff_points[0].points, 6, "there should be 6 points {:?}", &diff_points);
    assert_eq!(diff_points[0].total, 7, "there should be total 7 - {:?}", &diff_points);

    assert_eq!(
        diff_points[0].diffs,
        &[
            DiffPoint { interval: now - 480, value1: None, value2: Some(480.0) },
            DiffPoint { interval: now - 360, value1: None, value2: Some(360.0) },
            DiffPoint { interval: now - 300, value1: Some(300.0), value2: Some(3000.0) },
            DiffPoint { interval: now - 180, value1: Some(180.0), value2: None },
            DiffPoint { interval: now - 120, value1: None, value2: Some(120.0) },
            DiffPoint { interval: now - 60, value1: Some(60.0), value2: None },
        ]
    );

    Ok(())
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_diff_error() -> Result<(), Error> {
    let temp_dir = get_temp_dir();

    let path1 = get_file_path(&temp_dir, "diff1_1");
    let path2 = get_file_path(&temp_dir, "diff2_2");

    let now = 1528240800;

    let _file1 = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 60, points: 10 })
        .build(&path1)?;

    let _file2 = WhisperBuilder::default()
        .add_retention(Retention { seconds_per_point: 60, points: 11 })
        .build(&path2)?;

    let diff_result = whisper::diff::diff(&path1, &path2, false, now, now);

    assert!(diff_result.is_err());

    Ok(())
}
