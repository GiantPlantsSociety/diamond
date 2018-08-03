#![cfg(test)]

extern crate assert_cli;
extern crate whisper_tests;

use whisper_tests::*;

const NAME: &str = "whisper-set-aggregation-method";

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
        .with_args(&["invalid", "average"])
        .fails_with(1)
        .stderr().contains("No such file or directory (os error 2)")
        .execute()
}

#[test]
fn calling_with_invalid_method() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "unknown", "0.1" ])
        .fails_with(1)
        .stderr().contains("error: Invalid value for '<aggregationMethod>': Unsupported aggregation method 'unknown'")
        .execute()
}

#[test]
fn calling_with_invalid_xfactor() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "last", "nan" ])
        .fails_with(1)
        .stderr().contains("error: Invalid value for '<xFilesFactor>': invalid float literal")
        .execute()
}

#[test]
fn calling_with_last() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "last" ])
        .stdout().contains("Updated aggregation method")
        .stdout().contains("(average -> last)")
        .execute()
}

#[test]
fn calling_with_sum_and_xfactor() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "sum", "0.2" ])
        .stdout().contains("Updated aggregation method")
        .stdout().contains("(average -> sum)")
        .stdout().doesnt_contain("0.2")
        .execute()
}
