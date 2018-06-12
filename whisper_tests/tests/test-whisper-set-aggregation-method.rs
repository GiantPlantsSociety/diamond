extern crate assert_cli;
extern crate whisper_tests;

#[cfg(test)]
mod whisper_set_aggregation_method {
    use whisper_tests::*;

    const NAME: &str = "whisper-set-aggregation-method";

    #[test]
    fn calling_without_args() {
        get_binary_command(NAME)
            .fails_with(1)
            .stderr().contains("USAGE")
            .unwrap();
    }

    #[test]
    fn calling_help() {
        get_binary_command(NAME)
            .with_args(&["--help"])
            .stdout().contains("USAGE")
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_path() {
        get_binary_command(NAME)
            .with_args(&["invalid", "average"])
            .fails_with(1)
            .stderr().contains("No such file or directory (os error 2)")
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_method() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "unknown", "0.1" ])
            .fails_with(1)
            .stderr().contains("error: Invalid value for '<aggregationMethod>': Unsupported aggregation method 'unknown'")
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_xfactor() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "last", "nan" ])
            .fails_with(1)
            .stderr().contains("error: Invalid value for '<xFilesFactor>': invalid float literal")
            .unwrap();
    }

    #[test]
    fn calling_with_last() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "last" ])
            .stdout().contains("Updated aggregation method")
            .stdout().contains("(average -> last)")
            .unwrap();
    }

    #[test]
    fn calling_with_sum_and_xfactor() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "sum", "0.2" ])
            .stdout().contains("Updated aggregation method")
            .stdout().contains("(average -> sum)")
            .stdout().doesnt_contain("0.2")
            .unwrap();
    }
}
