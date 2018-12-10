use std::fs;
use byteorder::{BigEndian, WriteBytesExt};
use failure::Error;
use whisper::*;
use whisper::point::Point;
use whisper::retention::*;
use whisper_tests::*;

trait Dump {
    fn u32(self, value: u32) -> Self;
    fn f32(self, value: f32) -> Self;
    fn f64(self, value: f64) -> Self;
}

impl Dump for Vec<u8> {
    fn u32(mut self, value: u32) -> Self {
        self.write_u32::<BigEndian>(value).unwrap();
        self
    }

    fn f32(mut self, value: f32) -> Self {
        self.write_f32::<BigEndian>(value).unwrap();
        self
    }

    fn f64(mut self, value: f64) -> Self {
        self.write_f64::<BigEndian>(value).unwrap();
        self
    }
}

#[test]
fn test_update_snapshot() -> Result<(), Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "update_snapshot");

    {
        WhisperBuilder::default()
            .add_retention(Retention { seconds_per_point: 1, points: 10 })
            .build(path.clone())?;
    }

    let header = Vec::new()
        .u32(1) // aggregation method
        .u32(10) // max retention
        .f32(0.5) // x files factor
        .u32(1) // archives
        // archive info
        .u32(28) // offset
        .u32(1) // seconds per point
        .u32(10); // points

    assert_eq!(fs::read(&path)?,
        header.clone()
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b00, value: 123.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            .u32(0x5b171b00).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b01, value: 123.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            .u32(0x5b171b00).f64(123.0)
            .u32(0x5b171b01).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b04, value: 123.0 }, Point { interval: 0x5b171b06, value: 123.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            .u32(0x5b171b00).f64(123.0)
            .u32(0x5b171b01).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0x5b171b04).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0x5b171b06).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    Ok(())
}

#[test]
fn test_update_and_aggregate_snapshot() -> Result<(), Error> {
    let temp_dir = get_temp_dir();
    let path = get_file_path(&temp_dir, "update_and_aggregate_snapshot");

    {
        WhisperBuilder::default()
            .add_retention(Retention { seconds_per_point: 1, points: 5 })
            .add_retention(Retention { seconds_per_point: 2, points: 10 })
            .build(path.clone())?;
    }

    let header = Vec::new()
        .u32(1) // aggregation method
        .u32(20) // max retention
        .f32(0.5) // x files factor
        .u32(2) // archives
        // archive info
        .u32(40) // offset
        .u32(1) // seconds per point
        .u32(5) // points
        // archive info
        .u32(100) // offset
        .u32(2) // seconds per point
        .u32(10); // points

    assert_eq!(fs::read(&path)?,
        header.clone()
            // archive 1
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            // archive 2
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b00, value: 123.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            // archive 1
            .u32(0x5b171b00).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            // archive 2
            .u32(0x5b171b00).f64(123.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b01, value: 23.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            // archive 1
            .u32(0x5b171b00).f64(123.0)
            .u32(0x5b171b01).f64(23.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            // archive 2
            .u32(0x5b171b00).f64((123.0 + 23.0) / 2.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    {
        let mut file = WhisperFile::open(&path)?;
        file.update_many(&[Point { interval: 0x5b171b04, value: 123.0 }, Point { interval: 0x5b171b06, value: 1000.0 }], 0x5b171b04)?;
    }

    assert_eq!(fs::read(&path)?,
        header.clone()
            // archive 1
            .u32(0x5b171b00).f64(123.0)
            .u32(0x5b171b06).f64(1000.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0x5b171b04).f64(123.0)
            // archive 2
            .u32(0x5b171b00).f64((123.0 + 23.0) / 2.0)
            .u32(0).f64(0.0)
            .u32(0x5b171b04).f64(123.0)
            .u32(0x5b171b06).f64(1000.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
            .u32(0).f64(0.0)
    );

    Ok(())
}
