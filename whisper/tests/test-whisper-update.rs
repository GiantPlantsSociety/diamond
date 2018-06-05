extern crate assert_cli;

#[cfg(test)]
mod whisper_update {
    use assert_cli;

    const NAME: &str = "whisper-update";

    #[test]
    fn calling_without_args() {
        assert_cli::Assert::cargo_binary(NAME)
            .fails_with(1)
            .stderr().contains("USAGE")
            .unwrap();
    }

    #[test]
    fn calling_help() {
        assert_cli::Assert::cargo_binary(NAME)
            .with_args(&["--help"])
            .stdout().contains("USAGE")
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_path() {
        assert_cli::Assert::cargo_binary(NAME)
            .with_args(&["invalid", "1:1"])
            .fails_with(1)
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_timestamp() {
        assert_cli::Assert::cargo_binary(NAME)
            .with_args(&["some", "nottimestamp:1"])
            .fails_with(1)
            .unwrap();
    }

    #[test]
    fn calling_with_invalid_value() {
        assert_cli::Assert::cargo_binary(NAME)
            .with_args(&["some", "1:value"])
            .fails_with(1)
            .unwrap();
    }

}
