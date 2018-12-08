#![cfg(test)]
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

const NAME: &str = "whisper-create";

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
        .args(&["invalid/path", "60:1440"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No such file or directory (os error 2)").from_utf8());
}

#[test]
fn calling_with_invalid_method() {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()
        .unwrap()
        .path()
        .to_path_buf();

    let error =
        "error: Invalid value for '--aggregationMethod <aggregation_method>': Unsupported aggregation method 'unknown'";

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[
            path.to_str().unwrap(),
            "60:1440",
            "--aggregationMethod",
            "unknown",
        ]).assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
}

#[test]
fn calling_with_invalid_xfactor() {
    let path = Builder::new()
        .prefix("whisper")
        .suffix("info.wsp")
        .tempfile()
        .unwrap()
        .path()
        .to_path_buf();

    let error = "error: Invalid value for '--xFilesFactor <x_files_factor>': invalid float literal";

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "60:1440", "--xFilesFactor", "nan"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error).from_utf8());
}

#[test]
fn calling_creating_simple() {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "60:1440"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created: ").from_utf8())
        .stdout(predicate::str::contains(path.to_str().unwrap()).from_utf8())
        .stdout(predicate::str::contains("(17308 bytes)").from_utf8())
        .stderr("");
}

#[test]
fn calling_creating_multiple_retention() {
    let filename = "info.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "60:1440", "300:1440"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created: ").from_utf8())
        .stdout(predicate::str::contains(path.to_str().unwrap()).from_utf8())
        .stdout(predicate::str::contains("(34600 bytes)").from_utf8())
        .stderr("");
}

#[test]
fn calling_creating_with_present_file() {
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
        .args(&[path.to_str().unwrap(), "60:1440"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("File exists (os error 17)").from_utf8());
}
