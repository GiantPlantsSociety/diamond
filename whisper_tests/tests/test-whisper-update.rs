#![cfg(test)]

extern crate assert_cli;
extern crate whisper_tests;

use whisper_tests::*;

const NAME: &str = "whisper-update";

#[test]
fn calling_without_args() -> Result<(), assert_cli::AssertionError> {
    get_binary_command(NAME)
        .fails_with(1)
        .stderr().contains("USAGE")
        .execute()
}

#[test]
fn calling_help() -> Result<(), assert_cli::AssertionError> {
    get_binary_command(NAME)
        .with_args(&["--help"])
        .stdout().contains("USAGE")
        .execute()
}

#[test]
fn calling_with_invalid_path() -> Result<(), assert_cli::AssertionError> {
    get_binary_command(NAME)
        .with_args(&["invalid", "1:1"])
        .fails_with(1)
        .execute()
}

#[test]
fn calling_with_invalid_timestamp() -> Result<(), assert_cli::AssertionError> {
    get_binary_command(NAME)
        .with_args(&["some", "nottimestamp:1"])
        .fails_with(1)
        .execute()
}

#[test]
fn calling_with_invalid_value() -> Result<(), assert_cli::AssertionError> {
    get_binary_command(NAME)
        .with_args(&["some", "1:value"])
        .fails_with(1)
        .execute()
}
