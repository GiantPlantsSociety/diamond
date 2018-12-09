use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-set-aggregation-method";

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
        .args(&["invalid", "average"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No such file or directory (os error 2)").from_utf8());
    Ok(())
}

#[test]
fn calling_with_invalid_method() -> Result<(), Box<Error>> {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()?
        .path()
        .to_path_buf();

    let error =
        "error: Invalid value for '<aggregationMethod>': Unsupported aggregation method 'unknown'";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "unknown", "0.1"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_with_invalid_xfactor() -> Result<(), Box<Error>> {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()?
        .path()
        .to_path_buf();

    let error = "error: Invalid value for '<xFilesFactor>': invalid float literal";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "last", "nan"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_with_last() -> Result<(), Box<Error>> {
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
        .args(&[path.to_str().unwrap(), "last"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated aggregation method").from_utf8())
        .stdout(predicate::str::contains("(average -> last)").from_utf8());
    Ok(())
}

#[test]
fn calling_with_sum_and_xfactor() -> Result<(), Box<Error>> {
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
        .args(&[path.to_str().unwrap(), "sum", "0.2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated aggregation method").from_utf8())
        .stdout(predicate::str::contains("(average -> sum)").from_utf8())
        .stdout(predicate::str::contains("0.2").not().from_utf8());
    Ok(())
}
