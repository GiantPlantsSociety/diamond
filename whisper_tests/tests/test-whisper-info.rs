#![cfg(test)]

extern crate assert_cli;
extern crate unindent;
extern crate whisper_tests;

use whisper_tests::*;
use unindent::unindent;

const NAME: &str = "whisper-info";

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
        .stderr().contains("No such file or directory (os error 2)")
        .execute()
}

#[test]
fn calling_as_plain_for_unknown() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "unknown" ])
        .fails_with(1)
        .stderr().contains("Unknown field \"unknown\". Valid fields are maxRetention, xFilesFactor, aggregationMethod, archives, fileSize")
        .execute()
}

#[test]
fn calling_as_plain_for_max_retention() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "maxRetention" ])
        .stdout().contains("172800")
        .execute()
}

#[test]
fn calling_as_plain_for_x_files_factor() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "xFilesFactor" ])
        .stdout().contains("0.5")
        .execute()
}

#[test]
fn calling_as_plain_for_aggregation_method() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "aggregationMethod" ])
        .stdout().contains("average")
        .execute()
}

#[test]
fn calling_as_plain_for_file_size() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "fileSize" ])
        .stdout().contains("34600")
        .execute()
}

#[test]
fn calling_as_plain() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap() ])
        .stdout().contains(
            unindent("
                maxRetention: 172800
                xFilesFactor: 0.5
                aggregationMethod: average
                fileSize: 34600
                ").as_str()
            )
        .stdout().contains(
            unindent("
                Archive 0
                retention: 86400
                secondsPerPoint: 60
                points: 1440
                size: 17280
                offset: 40
                ").as_str()
            )
        .stdout().contains(
            unindent("
                Archive 1
                retention: 172800
                secondsPerPoint: 120
                points: 1440
                size: 17280
                offset: 17320
                ").as_str()
            )
        .execute()
}

#[test]
fn calling_as_json() -> Result<(), assert_cli::AssertionError> {
    let temp_dir = get_temp_dir();
    let path = copy_test_file(&temp_dir, "info.wsp");

    get_binary_command(NAME)
        .with_args(&[ path.to_str().unwrap(), "--json" ])
        .stdout().contains(
            unindent(r#"
                {
                  "aggregationMethod": "average",
                  "archives": [
                    {
                      "offset": 40,
                      "points": 1440,
                      "retention": 86400,
                      "secondsPerPoint": 60,
                      "size": 17280
                    },
                    {
                      "offset": 17320,
                      "points": 1440,
                      "retention": 172800,
                      "secondsPerPoint": 120,
                      "size": 17280
                    }
                  ],
                  "fileSize": 34600,
                  "maxRetention": 172800,
                  "xFilesFactor": 0.5
                }
                "#).as_str()
            )
        .execute()
}
