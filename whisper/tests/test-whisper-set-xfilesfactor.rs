use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-set-xfilesfactor";

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
        .args(&["invalid", "0.5"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error_msg).from_utf8());

    Ok(())
}

#[test]
fn calling_with_invalid_param() -> Result<(), Box<dyn Error>> {
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
        .args(&[path.to_str().unwrap(), "2.0"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Bad x_files_factor 2").from_utf8());
    Ok(())
}

#[test]
fn calling_with_fractional_number() -> Result<(), Box<dyn Error>> {
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
        .args(&[path.to_str().unwrap(), "0.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated xFilesFactor").from_utf8())
        .stdout(predicate::str::contains("(0.5 -> 0.1)").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_with_whole_param() -> Result<(), Box<dyn Error>> {
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
        .args(&[path.to_str().unwrap(), "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated xFilesFactor").from_utf8())
        .stdout(predicate::str::contains("(0.5 -> 1)").from_utf8())
        .stderr("");
    Ok(())
}
