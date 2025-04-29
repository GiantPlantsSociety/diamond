use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

const NAME: &str = "find-corrupt-whisper-files";

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
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").from_utf8())
        .stderr("");
    Ok(())
}

#[test]
fn calling_with_invalid_path() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .args(["invalid"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("invalid is not a directory or not exist!").from_utf8());

    Ok(())
}
