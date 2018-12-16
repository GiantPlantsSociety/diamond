use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-create";

#[test]
fn calling_without_args() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("USAGE").from_utf8());
    Ok(())
}

#[test]
fn calling_help() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_with_invalid_path() -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    let error_msg = "No such file or directory (os error 2)";
    #[cfg(windows)]
    let error_msg = "The system cannot find the path specified. (os error 3)";

    Command::cargo_bin(NAME)?
        .args(&["invalid/path", "60:1440"])
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

    let error =
        "error: Invalid value for '--aggregationMethod <aggregation_method>': Unsupported aggregation method 'unknown'";

    Command::cargo_bin(NAME)?
        .args(&[
            path.to_str().unwrap(),
            "60:1440",
            "--aggregationMethod",
            "unknown",
        ]).assert()
        .code(1)
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

    let error = "error: Invalid value for '--xFilesFactor <x_files_factor>': invalid float literal";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "60:1440", "--xFilesFactor", "nan"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
    Ok(())
}

#[test]
fn calling_creating_simple() -> Result<(), Box<dyn Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "60:1440"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created: ").from_utf8())
        .stdout(predicate::str::contains(path.to_str().unwrap()).from_utf8())
        .stdout(predicate::str::contains("(17308 bytes)").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_creating_multiple_retention() -> Result<(), Box<dyn Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "60:1440", "300:1440"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created: ").from_utf8())
        .stdout(predicate::str::contains(path.to_str().unwrap()).from_utf8())
        .stdout(predicate::str::contains("(34600 bytes)").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_creating_with_present_file() -> Result<(), Box<dyn Error>> {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()?
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path)?;

    #[cfg(unix)]
    let error_msg = "File exists (os error 17)";
    #[cfg(windows)]
    let error_msg = "The file exists. (os error 80)";

    Command::cargo_bin(NAME)?
        .args(&[path.to_str().unwrap(), "60:1440"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error_msg).from_utf8());

    Ok(())
}
