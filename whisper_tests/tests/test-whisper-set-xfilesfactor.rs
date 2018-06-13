extern crate assert_cli;
extern crate whisper_tests;

#[cfg(test)]
mod whisper_set_xfilesfactor {
    use whisper_tests::*;

    const NAME: &str = "whisper-set-xfilesfactor";

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
            .with_args(&["invalid", "0.5"])
            .fails_with(1)
            .stderr().contains("No such file or directory (os error 2)")
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_param() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "2.0" ])
            .fails_with(1)
            .stderr().contains("Bad x_files_factor 2")
            .unwrap();
    }

    #[test]
    fn calling_with_fractional_number() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "0.1" ])
            .stdout().contains("Updated xFilesFactor")
            .stdout().contains("(0.5 -> 0.1)")
            .unwrap();
    }

    #[test]
    fn calling_with_whole_param() {
        let temp_dir = get_temp_dir();
        let path = copy_test_file(&temp_dir, "info.wsp");

        get_binary_command(NAME)
            .with_args(&[ path.to_str().unwrap(), "1" ])
            .stdout().contains("Updated xFilesFactor")
            .stdout().contains("(0.5 -> 1)")
            .unwrap();
    }
}
