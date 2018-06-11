extern crate assert_cli;
extern crate whisper_tests;

#[cfg(test)]
mod whisper_dump {
    use whisper_tests::*;

    const NAME: &str = "whisper-dump";

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
            .with_args(&["invalid"])
            .fails_with(1)
            .unwrap();
    }
}