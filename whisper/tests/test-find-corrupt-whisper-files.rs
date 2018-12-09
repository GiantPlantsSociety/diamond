use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const NAME: &str = "find-corrupt-whisper-files";

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
        .args(&["invalid"])
        .assert()
        .code(1);
}
