use crate::rules::*;

/// Name of the file that stores formatting rules.
const CONFIG_FILE_NAME: &str = "shader-formatter.toml";

/// Represents a config file with formatting rules, deserialized from the disk.
pub struct Config {
    pub new_line_around_braces: NewLineAroundOpenBraceRule,
    pub indentation: IndentationRule,
    pub max_empty_lines: usize,
    pub local_variable_case: Option<Case>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_empty_lines: 1,
            new_line_around_braces: NewLineAroundOpenBraceRule::After,
            indentation: IndentationRule::FourSpaces,
            local_variable_case: None,
        }
    }
}

impl Config {
    /// Looks for a config file in the current directory or in parent directories.
    /// If not found returns an empty config as `Ok`, otherwise an error message.
    pub fn get() -> Result<Config, String> {
        // Get current directory.
        let mut current_dir = match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => return Err(error.to_string()),
        };

        loop {
            // Check if config exists in this directory.
            let path_to_config = current_dir.join(CONFIG_FILE_NAME);
            if path_to_config.exists() {
                return Self::load_from_file(path_to_config.as_path());
            }

            // Go to parent directory.
            current_dir = match current_dir.parent() {
                Some(p) => p.to_path_buf(),
                None => return Ok(Config::default()), // config not found, just return empty config
            }
        }
    }

    fn load_from_file(path_to_file: &std::path::Path) -> Result<Config, String> {
        // Read file.
        let file_content = match std::fs::read_to_string(path_to_file) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "failed to read the file at {}, error: {}",
                    path_to_file.display(),
                    e
                ))
            }
        };

        // Parse TOML.
        let table = match file_content.parse::<toml::Table>() {
            Ok(t) => t,
            Err(e) => {
                return Err(format!(
                    "failed to parse config file at {}, error: {}",
                    path_to_file.display(),
                    e
                ))
            }
        };

        let mut config = Config::default();
        for (key, value) in table {
            match key.as_str() {
                "Indentation" => {
                    // Make sure value is a string.
                    let value_str = match value.as_str() {
                        Some(s) => s,
                        None => {
                            return Err(format!(
                                "expected value for key \"{}\" to be a string",
                                key
                            ))
                        }
                    };

                    config.indentation = match value_str {
                        "Tab" => IndentationRule::Tab,
                        "TwoSpaces" => IndentationRule::TwoSpaces,
                        "FourSpaces" => IndentationRule::FourSpaces,
                        _ => {
                            return Err(format!(
                                "found unknown value \"{}\" for rule \"{}\"",
                                value_str, key
                            ))
                        }
                    };
                }
                "LocalVariableCase" => {
                    // Make sure value is a string.
                    let value_str = match value.as_str() {
                        Some(s) => s,
                        None => {
                            return Err(format!(
                                "expected value for key \"{}\" to be a string",
                                key
                            ))
                        }
                    };

                    config.local_variable_case = Some(match value_str {
                        "Camel" => Case::Camel,
                        "Pascal" => Case::Pascal,
                        "Snake" => Case::Snake,
                        "UpperSnake" => Case::UpperSnake,
                        _ => {
                            return Err(format!(
                                "found unknown value \"{}\" for rule \"{}\"",
                                value_str, key
                            ))
                        }
                    })
                }
                "NewLineAroundOpenBraceRule" => {
                    // Make sure value is a string.
                    let value_str = match value.as_str() {
                        Some(s) => s,
                        None => {
                            return Err(format!(
                                "expected value for key \"{}\" to be a string",
                                key
                            ))
                        }
                    };

                    config.new_line_around_braces = match value_str {
                        "After" => NewLineAroundOpenBraceRule::After,
                        "Before" => NewLineAroundOpenBraceRule::Before,
                        _ => {
                            return Err(format!(
                                "found unknown value \"{}\" for rule \"{}\"",
                                value_str, key
                            ))
                        }
                    }
                }
                "MaxEmptyLines" => {
                    // Make sure value is an integer.
                    let value_int = match value.as_integer() {
                        Some(s) => s,
                        None => {
                            return Err(format!(
                                "expected value for key \"{}\" to be a string",
                                key
                            ))
                        }
                    };

                    config.max_empty_lines = value_int as usize;
                }
                _ => return Err(format!("found unknown rule \"{}\"", key)),
            }
        }

        Ok(config)
    }
}
