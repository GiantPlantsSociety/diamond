use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-set-aggregation-method";

#[test]
fn calling_without_args() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("Usage").from_utf8());
    Ok(())
}

#[test]
fn calling_help() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_with_invalid_path() -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    let error_msg = "No such file or directory (os error 2)";
    #[cfg(windows)]
    let error_msg = "The system cannot find the file specified. (os error 2)";
    Command::cargo_bin(NAME)?
        .args(&["invalid", "average"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error_msg).from_utf8());

    Ok(())
}

#[test]
fn calling_with_invalid_method() -> Result<(), Box<dyn Error>> {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()?
        .path()
        .to_path_buf();

    let error = "error: invalid value 'unknown' for '[aggregationMethod]': Unsupported aggregation method 'unknown'";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "unknown", "0.1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_with_invalid_xfactor() -> Result<(), Box<dyn Error>> {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()?
        .path()
        .to_path_buf();

    let error = "error: invalid value 'nano' for '[xFilesFactor]': invalid float literal";

    // TODO: validate nan
    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "last", "nano"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_with_last() -> Result<(), Box<dyn Error>> {
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
fn calling_with_sum_and_xfactor() -> Result<(), Box<dyn Error>> {
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
