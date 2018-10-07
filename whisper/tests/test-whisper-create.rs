#![cfg(test)]

extern crate assert_cmd;
extern crate predicates;
extern crate tempfile;

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
