#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        config::Config,
        formatter::{Formatter, CHANGES_REQUIRED_ERR_MSG},
        rules::{Case, NewLineAroundOpenBraceRule},
    };

    fn get_project_root() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();

        loop {
            // Check if cargo exists in this directory.
            let test_path = path.join("Cargo.lock");
            if test_path.exists() {
                return path;
            }

            // Go to parent directory.
            path = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => panic!(),
            }
        }
    }

    fn compare_files_in_directory(config: Config, test_dir: &str) {
        let formatter = Formatter::new(config);

        let path_to_res = get_project_root().join("tests").join(test_dir);
        let path_to_input = path_to_res.join("input.hlsl");
        let path_to_output = path_to_res.join("output.hlsl");

        assert!(path_to_input.exists());
        assert!(path_to_output.exists());
        assert!(!path_to_input.is_dir());
        assert!(!path_to_output.is_dir());

        let input = std::fs::read_to_string(path_to_input).unwrap();
        let output = std::fs::read_to_string(path_to_output).unwrap();

        let result = match formatter.format(&input) {
            Ok(s) => s,
            Err(msg) => {
                panic!("{}", msg);
            }
        };

        assert_eq!(result, output);
    }

    fn test_complex_rules(config: Config, test_dir: &str) {
        let formatter = Formatter::new(config);

        let path_to_res = get_project_root().join("tests").join(test_dir);
        let path_to_fail = path_to_res.join("fail.hlsl");
        let path_to_success = path_to_res.join("success.hlsl");

        assert!(path_to_fail.exists());
        assert!(!path_to_fail.is_dir());
        assert!(path_to_success.exists());
        assert!(!path_to_success.is_dir());

        // Test fail.
        {
            let input = std::fs::read_to_string(path_to_fail).unwrap();

            match formatter.format(&input) {
                Ok(_) => panic!("expected the test to fail"),
                Err(msg) => assert!(msg.starts_with(CHANGES_REQUIRED_ERR_MSG)),
            }
        }

        // Test success.
        {
            let input = std::fs::read_to_string(path_to_success).unwrap();

            match formatter.format(&input) {
                Ok(_) => {}
                Err(msg) => panic!("{}", msg),
            }
        }
    }

    #[test]
    fn default_settings() {
        compare_files_in_directory(Config::default(), "default_settings");
    }

    #[test]
    fn new_line_before_brace() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.new_line_around_braces == NewLineAroundOpenBraceRule::After);

        // Change the setting.
        config.new_line_around_braces = NewLineAroundOpenBraceRule::Before;

        // Test.
        compare_files_in_directory(config, "new_line_before_brace");
    }

    #[test]
    fn spaces_in_brackets() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.new_line_around_braces == NewLineAroundOpenBraceRule::After);

        // Change the setting.
        config.spaces_in_brackets = true;

        // Test.
        compare_files_in_directory(config, "spaces_in_brackets");
    }

    #[test]
    fn variable_case() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.local_variable_case.is_none());

        // Change the setting.
        config.local_variable_case = Some(Case::Camel);

        // Test.
        test_complex_rules(config, "variable_case");
    }

    #[test]
    fn variable_prefix() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.bool_prefix.is_none());
        assert!(config.int_prefix.is_none());
        assert!(config.float_prefix.is_none());

        // Change the setting.
        config.bool_prefix = Some(String::from("b"));
        config.int_prefix = Some(String::from("i"));
        config.float_prefix = Some(String::from("f"));

        // Test.
        test_complex_rules(config.clone(), "variable_prefix/bool");
        test_complex_rules(config.clone(), "variable_prefix/int");
        test_complex_rules(config, "variable_prefix/float");
    }
}
