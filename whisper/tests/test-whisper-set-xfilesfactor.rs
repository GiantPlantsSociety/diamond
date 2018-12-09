use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-set-xfilesfactor";

#[test]
fn calling_without_args() {
    Command::cargo_bin(NAME)
        .unwrap()
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("USAGE").from_utf8());
}

#[test]
fn calling_help() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE").from_utf8())
        .stderr("");
}

#[test]
fn calling_with_invalid_path() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["invalid", "0.5"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No such file or directory (os error 2)").from_utf8());
}

#[test]
fn calling_with_invalid_param() {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path).unwrap();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "2.0"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Bad x_files_factor 2").from_utf8());
}

#[test]
fn calling_with_fractional_number() {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path).unwrap();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "0.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated xFilesFactor").from_utf8())
        .stdout(predicate::str::contains("(0.5 -> 0.1)").from_utf8())
        .stderr("");
}

#[test]
fn calling_with_whole_param() {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path).unwrap();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated xFilesFactor").from_utf8())
        .stdout(predicate::str::contains("(0.5 -> 1)").from_utf8())
        .stderr("");
}
