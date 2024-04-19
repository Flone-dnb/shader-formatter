use chumsky::span::SimpleSpan;
use convert_case::Casing;

use crate::{
    config::Config,
    parser::{self, Statement::*},
    rules::{Case, IndentationRule, NewLineAroundOpenBraceRule},
};

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

/// Applies rules on files.
pub struct Formatter {}

impl Formatter {
    pub fn new() -> Self {
        Self {}
    }

    /// Applies the most simplest formatting rules that do not require
    /// any prior parsing (no tokens required).
    pub fn apply_simple_rules(&self, config: &Config, content: &str) -> String {
        // Prepare indentation text.
        let indentation_text = match config.indentation {
            IndentationRule::Tab => "\t",
            IndentationRule::TwoSpaces => "  ",
            IndentationRule::FourSpaces => "    ",
        };

        let mut output = String::with_capacity(content.len());

        // Prepare some handy variables.
        let mut nesting_count: usize = 0;
        let mut consecutive_empty_new_line_count: usize = 0;
        let mut is_on_new_line = true;
        let mut ignore_until_text = false;

        for _char in content.chars() {
            // Just ignore '\r's.
            if _char == '\r' {
                continue;
            }

            // Handle new line.
            if _char == '\n' {
                is_on_new_line = true;

                if !ignore_until_text && consecutive_empty_new_line_count <= config.max_empty_lines
                {
                    output += LINE_ENDING;
                    output += &indentation_text.repeat(nesting_count);
                    consecutive_empty_new_line_count += 1;
                }

                continue;
            }

            if is_on_new_line {
                // Find where text starts.
                if _char != ' ' && _char != '\t' {
                    is_on_new_line = false;
                    ignore_until_text = false;
                    consecutive_empty_new_line_count = 0;
                } else {
                    continue;
                }
            }

            if ignore_until_text {
                if _char != ' ' && _char != '\t' {
                    ignore_until_text = false;
                } else {
                    continue;
                }
            }

            if _char == '{' {
                // Remove everything until text.
                let mut chars_to_remove = 0;
                for check in output.chars().rev() {
                    if check != ' ' && check != '\t' && check != '\n' && check != '\r' {
                        break;
                    }
                    chars_to_remove += 1;
                }
                for _ in 0..chars_to_remove {
                    output.pop();
                }

                // Handle new line.
                match config.new_line_around_braces {
                    NewLineAroundOpenBraceRule::After => {
                        // Add brace.
                        output.push(' ');
                        output.push(_char);

                        // Increase nesting.
                        nesting_count += 1;

                        // Insert a new line.
                        is_on_new_line = true;
                        output += LINE_ENDING;
                        output += &indentation_text.repeat(nesting_count);
                        consecutive_empty_new_line_count += 1;
                    }
                    NewLineAroundOpenBraceRule::Before => {
                        // Insert a new line.
                        is_on_new_line = true;
                        output += LINE_ENDING;
                        output += &indentation_text.repeat(nesting_count);

                        // Add brace.
                        output.push(_char);

                        // Add new line with increased nesting.
                        nesting_count += 1;
                        output += LINE_ENDING;
                        output += &indentation_text.repeat(nesting_count);
                        consecutive_empty_new_line_count += 1;
                    }
                }

                // Ignore everything until we find a text.
                ignore_until_text = true;
            } else if _char == '}' {
                // Decrease nesting.
                nesting_count = nesting_count.saturating_sub(1);

                // Remove everything until text.
                let mut chars_to_remove = 0;
                for check in output.chars().rev() {
                    if check != ' ' && check != '\t' && check != '\n' && check != '\r' {
                        break;
                    }
                    chars_to_remove += 1;
                }
                for _ in 0..chars_to_remove {
                    output.pop();
                }

                // Add a new line.
                output += LINE_ENDING;
                output += &indentation_text.repeat(nesting_count);

                // Copy brace.
                output.push(_char);

                // Add a new line.
                output += LINE_ENDING;
                output += &indentation_text.repeat(nesting_count);
                consecutive_empty_new_line_count += 1;
            } else if _char == '<' || _char == '[' || _char == '(' {
                output.push(_char);

                // Add space if needed.
                if config.spaces_in_brackets {
                    output.push(' ');
                }

                // Wait for text.
                ignore_until_text = true;
            } else if _char == '>' || _char == ']' || _char == ')' {
                // Remove everything until text.
                let mut chars_to_remove = 0;
                for check in output.chars().rev() {
                    if check != ' ' && check != '\t' && check != '\n' && check != '\r' {
                        break;
                    }
                    chars_to_remove += 1;
                }
                for _ in 0..chars_to_remove {
                    output.pop();
                }

                // Add space if needed.
                let nothing_in_brackets = match output.chars().last() {
                    None => false,
                    Some(c) => c == '<' || c == '[' || c == '(',
                };
                if config.spaces_in_brackets && !nothing_in_brackets {
                    output.push(' ');
                }

                output.push(_char);
            } else {
                if _char == ')' {
                    // Check if we have spaces like `(    )` to remove them.
                    let mut chars_to_remove = 0;
                    for check in output.chars().rev() {
                        if check != ' ' && check != '\t' {
                            break;
                        }
                        chars_to_remove += 1;
                    }
                    for _ in 0..chars_to_remove {
                        output.pop();
                    }
                }

                // Just copy the char.
                output.push(_char);
            }
        }

        output
    }

    /// Checks complex formatting rules that require prior parsing (tokens required).
    pub fn check_complex_rules(
        &self,
        config: &Config,
        statements: Vec<(parser::Statement<'_>, SimpleSpan)>,
    ) -> Result<(), String> {
        for (statement, _) in statements {
            match statement {
                VariableDeclaration(_, name) if config.local_variable_case.is_some() => {
                    match Self::is_case_different(name, config.local_variable_case.unwrap()) {
                        Ok(_) => {}
                        Err(correct) => {
                            return Err(format!(
                                "variable \"{}\" has incorrect case, the correct case is \"{}\"",
                                name, correct
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Checks if the case of the specified test string is different from the specified case.
    ///
    /// # Returns
    /// `Ok` if case is correct, otherwise `Err` that contains the specified string in the correct
    /// casing.
    fn is_case_different(test: &str, target_case: Case) -> Result<(), String> {
        let converted_str = match target_case {
            Case::Camel => test.to_case(convert_case::Case::Camel),
            Case::Snake => test.to_case(convert_case::Case::Snake),
            Case::Pascal => test.to_case(convert_case::Case::Pascal),
            Case::UpperSnake => test.to_case(convert_case::Case::UpperSnake),
        };

        if test != converted_str {
            return Err(converted_str);
        }

        Ok(())
    }
}
