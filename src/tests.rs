#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{config::Config, formatter::Formatter};

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

    #[test]
    fn simple_formatting_with_default_settings() {
        let config = Config::default();
        let formatter = Formatter::new();

        let path_to_res = get_project_root().join("tests").join("default_settings");
        let path_to_input = path_to_res.join("input.hlsl");
        let path_to_output = path_to_res.join("output.hlsl");

        assert!(path_to_input.exists());
        assert!(path_to_output.exists());
        assert!(!path_to_input.is_dir());
        assert!(!path_to_output.is_dir());

        let input = std::fs::read_to_string(path_to_input).unwrap();
        let output = std::fs::read_to_string(path_to_output).unwrap();

        let result = formatter.apply_simple_rules(&config, &input);

        assert_eq!(result, output);
    }
}
