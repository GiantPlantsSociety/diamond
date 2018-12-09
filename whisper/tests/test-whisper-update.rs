use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const NAME: &str = "whisper-update";

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
        .args(&["invalid", "1:1"])
        .assert()
        .code(1);
}

#[test]
fn calling_with_invalid_timestamp() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["invalid", "nottimestamp:1"])
        .assert()
        .code(1);
}

#[test]
fn calling_with_invalid_value() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["invalid", "1:value"])
        .assert()
        .code(1);
}
