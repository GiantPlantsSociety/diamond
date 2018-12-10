use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;
use unindent::unindent;

const NAME: &str = "whisper-info";

#[test]
fn calling_without_args() -> Result<(), Box<Error>> {
    Command::cargo_bin(NAME)?
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("USAGE").from_utf8());
    Ok(())
}

#[test]
fn calling_help() -> Result<(), Box<Error>> {
    Command::cargo_bin(NAME)?
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_with_invalid_path() -> Result<(), Box<Error>> {
    Command::cargo_bin(NAME)?
        .args(&["invalid"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No such file or directory (os error 2)").from_utf8());
    Ok(())
}

#[test]
fn calling_as_plain_for_unknown() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    let error = "Unknown field \"unknown\". Valid fields are maxRetention, xFilesFactor, aggregationMethod, archives, fileSize";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "unknown"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_as_plain_for_max_retention() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "maxRetention"])
        .assert()
        .success()
        .stdout(predicate::str::contains("172800").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_as_plain_for_x_files_factor() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "xFilesFactor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.5").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_as_plain_for_aggregation_method() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "aggregationMethod"])
        .assert()
        .success()
        .stdout(predicate::str::contains("average").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_as_plain_for_file_size() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "fileSize"])
        .assert()
        .success()
        .stdout(predicate::str::contains("34600").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_as_plain() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                unindent(
                    "
                maxRetention: 172800
                xFilesFactor: 0.5
                aggregationMethod: average
                fileSize: 34600
                ",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
                Archive 0
                retention: 86400
                secondsPerPoint: 60
                points: 1440
                size: 17280
                offset: 40
                ",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
                Archive 1
                retention: 172800
                secondsPerPoint: 120
                points: 1440
                size: 17280
                offset: 17320
                ",
                ).as_str(),
            ).from_utf8(),
        ).stderr("");
    Ok(())
}

#[test]
fn calling_as_json() -> Result<(), Box<Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                unindent(
                    r#"
                {
                  "aggregationMethod": "average",
                  "archives": [
                    {
                      "offset": 40,
                      "points": 1440,
                      "retention": 86400,
                      "secondsPerPoint": 60,
                      "size": 17280
                    },
                    {
                      "offset": 17320,
                      "points": 1440,
                      "retention": 172800,
                      "secondsPerPoint": 120,
                      "size": 17280
                    }
                  ],
                  "fileSize": 34600,
                  "maxRetention": 172800,
                  "xFilesFactor": 0.5
                }
                "#,
                ).as_str(),
            ).from_utf8(),
        ).stderr("");
    Ok(())
}
