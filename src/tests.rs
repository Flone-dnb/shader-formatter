#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        config::Config,
        formatter::{Formatter, CHANGES_REQUIRED_ERR_MSG},
        rules::{Case, NewLineOnOpenBrace},
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

        let result = match formatter.format(&input, false) {
            Ok(s) => s,
            Err(msg) => {
                panic!("{}", msg);
            }
        };

        assert_eq!(result, output);
    }

    fn test_formatting_fail_success(config: Config, test_dir: &str) {
        let formatter = Formatter::new(config);

        let path_to_res = get_project_root().join("tests").join(test_dir);

        let mut paths_to_fail = Vec::new();
        let mut paths_to_success = Vec::new();

        let path_to_fail = path_to_res.join("fail.hlsl");
        let path_to_success = path_to_res.join("success.hlsl");

        if !path_to_fail.exists() && !path_to_success.exists() {
            if path_to_res.join("fail1.hlsl").exists() {
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
            }

            if path_to_res.join("success1.hlsl").exists() {
                // Add success files.
                let mut test_file_number = 1usize;
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
            }
        } else {
            paths_to_fail.push(path_to_fail);
            paths_to_success.push(path_to_success);
        }

        assert!(!paths_to_fail.is_empty() || !paths_to_success.is_empty());

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
            let input = std::fs::read_to_string(path.clone()).unwrap();

            match formatter.format(&input, false) {
                Ok(_) => panic!("expected the test to fail (file {})", path.display()),
                Err(msg) => assert!(msg.starts_with(CHANGES_REQUIRED_ERR_MSG)),
            }
        }

        // Test success.
        for path in paths_to_success {
            let input = std::fs::read_to_string(path).unwrap();

            match formatter.format(&input, false) {
                Ok(_) => {}
                Err(msg) => panic!("{}", msg),
            }
        }
    }

    #[test]
    fn default_settings() {
        compare_files_in_directory(Config::default(), "default_settings/general");
    }

    #[test]
    fn add_indentation_on_new_line_in_braces() {
        let config = Config::default();

        assert!(config.spaces_in_brackets == false);

        compare_files_in_directory(
            config,
            "default_settings/add_indentation_on_new_line_in_braces",
        );
    }

    #[test]
    fn dont_remove_spaces_in_comparison() {
        let config = Config::default();

        assert!(config.spaces_in_brackets == false);

        compare_files_in_directory(config, "default_settings/dont_remove_spaces_in_comparison");
    }

    #[test]
    fn bracket_avoids_comments() {
        let config = Config::default();

        assert!(config.spaces_in_brackets == false);

        compare_files_in_directory(
            Config::default(),
            "default_settings/bracket_avoids_comments/1",
        );
        compare_files_in_directory(
            Config::default(),
            "default_settings/bracket_avoids_comments/2",
        );
        compare_files_in_directory(
            Config::default(),
            "default_settings/bracket_avoids_comments/3",
        );
    }

    #[test]
    fn long_line_split_indentation() {
        compare_files_in_directory(
            Config::default(),
            "default_settings/long_line_split_indentation",
        );
    }

    #[test]
    fn func_arg_input_modifier_parsing() {
        test_formatting_fail_success(
            Config::default(),
            "default_settings/func_arg_input_modifier_parsing",
        )
    }

    #[test]
    fn new_line_after_brace_in_multiline_macro() {
        let config = Config::default();

        assert!(config.new_line_around_braces == NewLineOnOpenBrace::After);

        compare_files_in_directory(
            config,
            "default_settings/new_line_after_brace_in_multiline_macro",
        );
    }

    #[test]
    fn new_line_before_brace_in_multiline_macro() {
        let mut config = Config::default();

        assert!(config.new_line_around_braces != NewLineOnOpenBrace::Before);
        config.new_line_around_braces = NewLineOnOpenBrace::Before;

        compare_files_in_directory(config, "new_line_before_brace_in_multiline_macro");
    }

    #[test]
    fn empty_files() {
        // Test.
        test_formatting_fail_success(Config::default(), "empty_files");
    }

    #[test]
    fn new_line_before_brace() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.new_line_around_braces == NewLineOnOpenBrace::After);

        // Change the setting.
        config.new_line_around_braces = NewLineOnOpenBrace::Before;

        // Test.
        compare_files_in_directory(config, "new_line_before_brace");
    }

    #[test]
    fn spaces_in_brackets() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.new_line_around_braces == NewLineOnOpenBrace::After);

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
        test_formatting_fail_success(config, "variable_case");
    }

    #[test]
    fn function_case() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.function_case.is_none());

        // Change the setting.
        config.function_case = Some(Case::Camel);

        // Test.
        test_formatting_fail_success(config, "function_case");
    }

    #[test]
    fn struct_case() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.struct_case.is_none());

        // Change the setting.
        config.struct_case = Some(Case::Pascal);

        // Test.
        test_formatting_fail_success(config, "struct_case");
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
        test_formatting_fail_success(config.clone(), "variable_prefix/bool");
        test_formatting_fail_success(config.clone(), "variable_prefix/int");
        test_formatting_fail_success(config, "variable_prefix/float");
    }

    #[test]
    fn global_variable_prefix() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.global_variable_prefix.is_none());
        assert!(config.int_prefix.is_none());
        assert!(config.variable_case.is_none());

        // Change the setting.
        config.global_variable_prefix = Some(String::from("g_"));
        config.int_prefix = Some(String::from("i"));
        config.variable_case = Some(Case::Camel);

        // Test.
        test_formatting_fail_success(config, "global_variable_prefix");
    }

    #[test]
    fn require_docs_on_functions() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(!config.require_docs_on_functions);

        // Change the setting.
        config.require_docs_on_functions = true;

        // Test.
        test_formatting_fail_success(config, "require_docs_on_functions");
    }

    #[test]
    fn require_docs_on_structs() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(!config.require_docs_on_structs);

        // Change the setting.
        config.require_docs_on_structs = true;

        // Test.
        test_formatting_fail_success(config, "require_docs_on_structs");
    }

    #[test]
    fn indent_preprocessor() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(!config.indent_preprocessor);

        // Change the setting.
        config.indent_preprocessor = true;

        // Test.
        compare_files_in_directory(config, "indent_preprocessor");
    }

    #[test]
    fn preprocessor_if_creates_nesting() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(!config.indent_preprocessor);
        assert!(!config.preprocessor_if_creates_nesting);

        // Change the setting.
        config.indent_preprocessor = true;
        config.preprocessor_if_creates_nesting = true;

        // Test.
        compare_files_in_directory(config.clone(), "preprocessor_if_creates_nesting/1");
        compare_files_in_directory(config, "preprocessor_if_creates_nesting/2");
    }

    #[test]
    fn nolint() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(config.variable_case.is_none());
        assert!(config.struct_case.is_none());

        // Change the setting.
        config.variable_case = Some(Case::Camel);
        config.struct_case = Some(Case::Pascal);

        // Test.
        test_formatting_fail_success(config, "nolint");
    }

    #[test]
    fn noformat() {
        // Test.
        compare_files_in_directory(Config::default(), "noformat/1");
        compare_files_in_directory(Config::default(), "noformat/2");
        compare_files_in_directory(Config::default(), "noformat/3");
        test_formatting_fail_success(Config::default(), "noformat/no_matching_end")
    }

    #[test]
    fn require_docs_on_fields() {
        let mut config = Config::default();

        // Make sure default config uses other setting.
        assert!(!config.require_docs_on_fields);

        // Change the setting.
        config.require_docs_on_fields = true;

        // Test.
        test_formatting_fail_success(config, "require_docs_on_fields");
    }

    #[test]
    fn empty_lines_no_indent() {
        let mut config = Config::default();

        config.indent_preprocessor = true;
        config.preprocessor_if_creates_nesting = true;

        // Test.
        compare_files_in_directory(config, "empty_lines_no_indent");
    }
}
