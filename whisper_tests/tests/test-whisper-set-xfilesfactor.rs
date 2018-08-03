#![cfg(test)]

extern crate assert_cli;
extern crate whisper_tests;

use whisper_tests::*;

const NAME: &str = "whisper-set-xfilesfactor";

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
        .with_args(&["invalid", "0.5"])
        .fails_with(1)
        .stderr().contains("No such file or directory (os error 2)")
        .execute()
}

#[test]
fn calling_with_invalid_param() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "2.0" ])
        .fails_with(1)
        .stderr().contains("Bad x_files_factor 2")
        .execute()
}

#[test]
fn calling_with_fractional_number() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "0.1" ])
        .stdout().contains("Updated xFilesFactor")
        .stdout().contains("(0.5 -> 0.1)")
        .execute()
}

#[test]
fn calling_with_whole_param() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "1" ])
        .stdout().contains("Updated xFilesFactor")
        .stdout().contains("(0.5 -> 1)")
        .execute()
}
