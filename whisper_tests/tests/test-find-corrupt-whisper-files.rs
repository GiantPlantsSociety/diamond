#![cfg(test)]

extern crate assert_cli;
extern crate whisper_tests;

use whisper_tests::*;

const NAME: &str = "find-corrupt-whisper-files";

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
        .with_args(&["invalid"])
        .fails_with(1)
        .execute()
}
