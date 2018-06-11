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
}
