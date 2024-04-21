use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use convert_case::Casing;

use crate::{
    config::Config,
    helpers,
    parser::{self, ComplexToken::*, Type},
    rules::{Case, IndentationRule, NewLineAroundOpenBraceRule},
};

/// Text that we append to the beginning of an error message if manual changes (in the code) are required
/// (like changing a variable's case).
pub const CHANGES_REQUIRED_ERR_MSG: &str = "changes required";

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

/// Applies rules on files.
pub struct Formatter {
    config: Config,
}

impl Formatter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Formats the specified content according to the formatting rules from config.
    ///
    /// # Return
    /// `Ok(String)` if successful with formatted content, otherise `Err(String)` with a meaningful
    /// error message.
    pub fn format(&self, content: &str) -> Result<String, String> {
        // Apply rules that don't need tokens.
        let output = self.apply_simple_rules(content);

        // Parse tokens.
        let (tokens, errors) = parser::token_parser()
            .parse(output.as_str())
            .into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            if let Some(error) = errors.into_iter().next() {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, output.as_str());
                let reason = error.reason();

                return Err(format!(
                    "token parser error at line {} column {}, reason: {}",
                    line, column, reason
                ));
            }
        }

        // Exit of no tokens returned (not an error).
        if tokens.is_none() {
            return Ok(output);
        }
        let tokens: Vec<(parser::Token<'_>, SimpleSpan)> = tokens.unwrap();

        // Parse statements.
        let (statements, errors) = parser::complex_token_parser()
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            if let Some(error) = errors.into_iter().next() {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, output.as_str());
                let reason = error.reason();
                return Err(format!(
                    "statement parser error at line {} column {}, reason: {}",
                    line, column, reason
                ));
            }
        }

        match statements {
            None => Ok(output), // nothing to do here
            Some(statements) => match self.check_complex_rules(statements) {
                Ok(_) => Ok(output), // everything is fine
                Err(msg) => Err(format!("{}: {}", CHANGES_REQUIRED_ERR_MSG, msg)),
            },
        }
    }

    /// Applies the most simplest formatting rules that do not require
    /// any prior parsing (no tokens required).
    fn apply_simple_rules(&self, content: &str) -> String {
        // Prepare indentation text.
        let indentation_text = match self.config.indentation {
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

                if !ignore_until_text
                    && consecutive_empty_new_line_count <= self.config.max_empty_lines
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
                match self.config.new_line_around_braces {
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

                // Don't insert a new line here, here is an example why:
                // struct Foo{
                // };
                // The `;` will be on the new line if we insert one.
            } else if _char == '<' || _char == '[' || _char == '(' {
                output.push(_char);

                // Add space if needed.
                if self.config.spaces_in_brackets {
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
                if self.config.spaces_in_brackets && !nothing_in_brackets {
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
    fn check_complex_rules(
        &self,
        statements: Vec<(parser::ComplexToken<'_>, SimpleSpan)>,
    ) -> Result<(), String> {
        for (statement, _) in statements {
            match statement {
                VariableDeclaration(_type, name) => {
                    self.check_variable_name(name, _type)?;
                }
                Struct(_, fields) => {
                    for (field_type, field_name) in fields {
                        self.check_variable_name(field_name, field_type)?;
                    }
                }
                Function(_, args) => {
                    for (arg_type, arg_name) in args {
                        self.check_variable_name(arg_name, arg_type)?;
                    }
                }
                Other(_) => {}
            }
        }

        Ok(())
    }

    /// Checks various complex formatting rules on the specified variable.
    ///
    /// # Return
    /// `Ok` if the name is correct (according to the rules), otherwise `Err` that container
    /// an error message with suggestions according to the rules.
    fn check_variable_name(&self, name: &str, _type: Type) -> Result<(), String> {
        // Check case.
        if self.config.variable_case.is_some() {
            match Self::is_case_different(name, self.config.variable_case.unwrap()) {
                Ok(_) => {}
                Err(correct) => {
                    return Err(format!(
                        "variable \"{}\" has incorrect case, the correct case is \"{}\"",
                        name, correct
                    ));
                }
            }
        }

        // Check prefixes.
        if _type == Type::Bool && self.config.bool_prefix.is_some() {
            Self::check_prefix(name, self.config.bool_prefix.as_ref().unwrap())?
        }
        if _type == Type::Integer && self.config.int_prefix.is_some() {
            Self::check_prefix(name, self.config.int_prefix.as_ref().unwrap())?
        }
        if _type == Type::Float && self.config.float_prefix.is_some() {
            Self::check_prefix(name, self.config.float_prefix.as_ref().unwrap())?
        }

        Ok(())
    }

    /// This function contains repetitive code for checking prefixes.
    ///
    /// # Return
    /// `Ok` if prefix is correct, otherwise `Err` that contains a meaningful error message
    /// about wrong prefix.
    fn check_prefix(name: &str, prefix: &str) -> Result<(), String> {
        if !name.starts_with(prefix) {
            return Err(format!(
                "variable \"{}\" has incorrect prefix, the correct prefix is \"{}\"",
                name, prefix
            ));
        }

        Ok(())
    }

    /// Checks if the case of the specified test string is different from the specified case.
    ///
    /// # Return
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
