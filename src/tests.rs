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

        let mut paths_to_fail = Vec::new();
        let mut paths_to_success = Vec::new();

        let path_to_fail = path_to_res.join("fail.hlsl");
        let path_to_success = path_to_res.join("success.hlsl");

        if !path_to_fail.exists() {
            assert!(
                path_to_res.join("fail1.hlsl").exists(),
                "`fail` file should exist or there must be `fail1`, `fail2` and etc files"
            );

            // Add fail files.
            let mut test_file_number = 1usize;
            loop {
                // Check if exists.
                let path = path_to_res.join(format!("fail{}.hlsl", test_file_number));
                if !path.exists() {
                    break;
                }

                // Add.
                paths_to_fail.push(path);
                test_file_number += 1;
            }

            // Add success files.
            test_file_number = 1usize;
            loop {
                // Check if exists.
                let path = path_to_res.join(format!("success{}.hlsl", test_file_number));
                if !path.exists() {
                    break;
                }

                // Add.
                paths_to_success.push(path);
                test_file_number += 1;
            }
        } else {
            paths_to_fail.push(path_to_fail);
            paths_to_success.push(path_to_success);
        }

        assert!(!paths_to_fail.is_empty() && !paths_to_success.is_empty());

        for path in &paths_to_fail {
            assert!(path.exists());
            assert!(!path.is_dir());
        }
        for path in &paths_to_success {
            assert!(path.exists());
            assert!(!path.is_dir());
        }

        // Test fail.
        for path in paths_to_fail {
            let input = std::fs::read_to_string(path).unwrap();

            match formatter.format(&input) {
                Ok(_) => panic!("expected the test to fail"),
                Err(msg) => assert!(msg.starts_with(CHANGES_REQUIRED_ERR_MSG)),
            }
        }

        // Test success.
        for path in paths_to_success {
            let input = std::fs::read_to_string(path).unwrap();

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
        assert!(config.variable_case.is_none());

        // Change the setting.
        config.variable_case = Some(Case::Camel);

        // Test.
        test_complex_rules(config, "variable_case");
    }

    #[test]
    fn function_case() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.function_case.is_none());

        // Change the setting.
        config.function_case = Some(Case::Camel);

        // Test.
        test_complex_rules(config, "function_case");
    }

    #[test]
    fn struct_case() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.struct_case.is_none());

        // Change the setting.
        config.struct_case = Some(Case::Pascal);

        // Test.
        test_complex_rules(config, "struct_case");
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
