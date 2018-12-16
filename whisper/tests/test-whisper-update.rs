use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

const NAME: &str = "whisper-update";

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
    let error_msg = "The system cannot find the file specified. (os error 2)";

    Command::cargo_bin(NAME)?
        .args(&["invalid", "1:1"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(error_msg).from_utf8());

    Ok(())
}

#[test]
fn calling_with_invalid_timestamp() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .args(&["invalid", "nottimestamp:1"])
        .assert()
        .code(1);
    Ok(())
}

#[test]
fn calling_with_invalid_value() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(NAME)?
        .args(&["invalid", "1:value"])
        .assert()
        .code(1);
    Ok(())
}
