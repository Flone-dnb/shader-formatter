use toml::Value;

use crate::rules::*;

/// Name of the file that stores formatting rules.
const CONFIG_FILE_NAME: &str = "shader-formatter.toml";

/// Represents a config file with formatting rules, deserialized from the disk.
#[derive(Clone)]
pub struct Config {
    pub new_line_around_braces: NewLineAroundOpenBraceRule,
    pub indentation: IndentationRule,
    pub max_empty_lines: usize,
    pub spaces_in_brackets: bool,
    pub variable_case: Option<Case>,
    pub function_case: Option<Case>,
    pub struct_case: Option<Case>,
    pub bool_prefix: Option<String>,
    pub int_prefix: Option<String>,
    pub float_prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_empty_lines: 1,
            new_line_around_braces: NewLineAroundOpenBraceRule::After,
            indentation: IndentationRule::FourSpaces,
            spaces_in_brackets: false,
            variable_case: None,
            function_case: None,
            struct_case: None,
            bool_prefix: None,
            int_prefix: None,
            float_prefix: None,
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
                    config.indentation = match Self::toml_value_to_string(&key, &value)? {
                        "Tab" => IndentationRule::Tab,
                        "TwoSpaces" => IndentationRule::TwoSpaces,
                        "FourSpaces" => IndentationRule::FourSpaces,
                        other => {
                            return Err(format!(
                                "found unknown value \"{}\" for rule \"{}\"",
                                other, key
                            ))
                        }
                    };
                }
                "VariableCase" => {
                    config.variable_case = Some(Self::toml_value_to_case(&key, &value)?)
                }
                "FunctionCase" => {
                    config.function_case = Some(Self::toml_value_to_case(&key, &value)?)
                }
                "StructCase" => config.struct_case = Some(Self::toml_value_to_case(&key, &value)?),
                "NewLineAroundOpenBraceRule" => {
                    config.new_line_around_braces = match Self::toml_value_to_string(&key, &value)?
                    {
                        "After" => NewLineAroundOpenBraceRule::After,
                        "Before" => NewLineAroundOpenBraceRule::Before,
                        other => {
                            return Err(format!(
                                "found unknown value \"{}\" for rule \"{}\"",
                                other, key
                            ))
                        }
                    }
                }
                "MaxEmptyLines" => {
                    config.max_empty_lines = Self::toml_value_to_usize(&key, &value)?;
                }
                "SpacesInBrackets" => {
                    config.spaces_in_brackets = Self::toml_value_to_bool(&key, &value)?;
                }
                "BoolPrefix" => {
                    config.bool_prefix =
                        Some(Self::toml_value_to_string(&key, &value)?.to_string());
                }
                "IntPrefix" => {
                    config.int_prefix = Some(Self::toml_value_to_string(&key, &value)?.to_string());
                }
                "FloatPrefix" => {
                    config.float_prefix =
                        Some(Self::toml_value_to_string(&key, &value)?.to_string());
                }
                _ => return Err(format!("found unknown rule \"{}\"", key)),
            }
        }

        Ok(config)
    }

    /// Tries to convert a TOML value to a case type and returns a meaningful error message
    /// if we failed.
    fn toml_value_to_case(key: &str, value: &Value) -> Result<Case, String> {
        match Self::toml_value_to_string(key, value)? {
            "Camel" => Ok(Case::Camel),
            "Pascal" => Ok(Case::Pascal),
            "Snake" => Ok(Case::Snake),
            "UpperSnake" => Ok(Case::UpperSnake),
            other => Err(format!(
                "found unknown value \"{}\" for rule \"{}\"",
                other, key
            )),
        }
    }

    /// Tries to convert a TOML value to a string and returns a meaningful error message
    /// in case we failed.
    fn toml_value_to_string<'a>(key: &str, value: &'a Value) -> Result<&'a str, String> {
        match value.as_str() {
            Some(v) => Ok(v),
            None => Err(format!("expected value for key \"{}\" to be a string", key)),
        }
    }

    /// Tries to convert a TOML value to a `usize` and returns a meaningful error message
    /// in case we failed.
    fn toml_value_to_usize(key: &str, value: &Value) -> Result<usize, String> {
        match value.as_integer() {
            Some(v) => {
                if v.is_negative() {
                    return Err(format!(
                        "expected value for key \"{}\" to be an unsigned integer",
                        key
                    ));
                }

                Ok(v as usize)
            }
            None => Err(format!(
                "expected value for key \"{}\" to be an integer",
                key
            )),
        }
    }

    /// Tries to convert a TOML value to a boolean and returns a meaningful error message
    /// in case we failed.
    fn toml_value_to_bool(key: &str, value: &Value) -> Result<bool, String> {
        match value.as_bool() {
            Some(v) => Ok(v),
            None => Err(format!(
                "expected value for key \"{}\" to be a boolean",
                key
            )),
        }
    }
}
