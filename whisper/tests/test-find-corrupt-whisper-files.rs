extern crate assert_cli;

#[cfg(test)]
mod find_corrupt_whisper_files {
    use assert_cli;

    const NAME: &str = "find-corrupt-whisper-files";

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
            .with_args(&["invalid"])
            .fails_with(1)
            .unwrap();
    }
}
