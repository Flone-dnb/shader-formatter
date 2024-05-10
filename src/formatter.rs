use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use convert_case::Casing;

use crate::{
    config::Config,
    helpers,
    parser::{self, ComplexToken::*, FunctionInfo, StructInfo, Token, Type},
    rules::{Case, IndentationRule, NewLineAroundOpenBraceRule},
};

/// Text that we append to the beginning of an error message if manual changes (in the code) are required
/// (like changing a variable's case).
pub const CHANGES_REQUIRED_ERR_MSG: &str = "changes required";

/// Comments used to tell the formatter to don't format (ignore) some lines of code.
const NOFORMAT_BEGIN_COMMENT: &str = " NOFORMATBEGIN";
const NOFORMAT_END_COMMENT: &str = " NOFORMATEND";

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
    /// # Arguments
    /// - `content` Text to format.
    /// - `print_tokens` Defines whether or not to print parsed token to stdout (used for debugging).
    ///
    /// # Return
    /// `Ok(String)` if successful with formatted content, otherise `Err(String)` with a meaningful
    /// error message.
    pub fn format(&self, content: &str, print_tokens: bool) -> Result<String, String> {
        // Exit on empty input.
        if content.is_empty() {
            return Ok(content.to_owned());
        }

        // Apply rules that don't need tokens.
        let output = self.apply_simple_rules(content);
        if let Err(msg) = output {
            return Err(format!("{}: {}", CHANGES_REQUIRED_ERR_MSG, msg));
        }
        let output = output.unwrap();

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

        // Print tokens if needed.
        if print_tokens {
            println!("parsed tokens:");
            for token in &tokens {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(token.1.start, content);
                println!("[line {}, column {}] {}", line, column, token.0);
            }
            println!("------------------------------------\n");
        }

        // Parse statements.
        let (complex_tokens, errors) = parser::complex_token_parser()
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

        match complex_tokens {
            None => Ok(output), // nothing to do here
            Some(tokens) => {
                // Print tokens if needed.
                if print_tokens {
                    println!("parsed complex tokens:");
                    for token in &tokens {
                        let (line, column) =
                            helpers::span_offset_to_line_and_column(token.1.start, content);
                        println!("[line {}, column {}] {}", line, column, token.0);
                    }
                    println!("------------------------------------\n");
                }

                // Check rules.
                match self.check_complex_rules(tokens) {
                    Ok(_) => Ok(output), // everything is fine
                    Err(msg) => Err(format!("{}: {}", CHANGES_REQUIRED_ERR_MSG, msg)),
                }
            }
        }
    }

    /// Applies the most simplest formatting rules that do not require
    /// any prior parsing (no tokens required).
    ///
    /// # Return
    /// `Ok` with formatted code or `Err` with an error message.
    fn apply_simple_rules(&self, content: &str) -> Result<String, String> {
        // Prepare indentation text.
        let indentation_text = match self.config.indentation {
            IndentationRule::Tab => "\t",
            IndentationRule::TwoSpaces => "  ",
            IndentationRule::FourSpaces => "    ",
        };

        let mut output = String::with_capacity(content.len());

        // Prepare some handy variables...

        // For nesting.
        let mut nesting_count: usize = 0;

        // For new lines.
        let mut consecutive_empty_new_line_count: usize = 0;
        let mut is_on_new_line = true;
        let mut ignore_until_text = false;
        let mut stop_ignoring_if_end_of_line = false;

        // For comments.
        let mut inside_c_comment_count: usize = 0;
        let mut inside_comment = false;
        let mut last_comment_line = String::new(); // contains last found line of comment

        // For preprocessor directives.
        let mut preproc_add_nesting_on_next_line = false;
        let mut line_started_with_preprocessor = false;

        // For macros.
        let mut last_non_space_char_is_backslash = false;
        let mut prev_line_ended_with_backslash = false;

        // Other.
        let mut last_3_chars = [' '; 3];
        let mut inside_no_format = false;

        for _char in content.chars() {
            // Just ignore '\r's.
            if _char == '\r' {
                continue;
            }

            // Handle new line.
            if _char == '\n' {
                is_on_new_line = true;

                if preproc_add_nesting_on_next_line {
                    nesting_count += 1;
                    preproc_add_nesting_on_next_line = false;
                }

                if (!ignore_until_text || stop_ignoring_if_end_of_line)
                    && consecutive_empty_new_line_count <= self.config.max_empty_lines
                {
                    ignore_until_text = false;
                    stop_ignoring_if_end_of_line = false;

                    output += LINE_ENDING;
                    output += &indentation_text.repeat(nesting_count);
                    consecutive_empty_new_line_count += 1;
                }

                continue;
            }

            if is_on_new_line {
                inside_comment = false;

                prev_line_ended_with_backslash = last_non_space_char_is_backslash;

                // Find where text starts.
                if _char != ' ' && _char != '\t' {
                    is_on_new_line = false;
                    ignore_until_text = false;
                    stop_ignoring_if_end_of_line = false;
                    consecutive_empty_new_line_count = 0;

                    if _char == '#' {
                        line_started_with_preprocessor = true;

                        if !self.config.indent_preprocessor {
                            // Remove everything until the beginning of the line.
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
                    } else {
                        line_started_with_preprocessor = false;

                        if _char == '=' {
                            // Since this is the first character on the line,
                            // add an additional indentation because this line is probably too long
                            // and was split into 2 lines, example:
                            // int some_long_variable_name
                            //     = ...;
                            output += indentation_text;
                        }
                    }

                    if inside_c_comment_count > 0 && _char == '*' {
                        // Add a single space for C-style comments to look good.
                        output.push(' ');
                    }
                } else {
                    continue;
                }
            }

            if ignore_until_text {
                if _char != ' ' && _char != '\t' {
                    ignore_until_text = false;
                    stop_ignoring_if_end_of_line = false;
                } else {
                    continue;
                }
            }

            // Detect a C-style comment.
            if last_3_chars[1] == '/' && last_3_chars[2] == '*' && (_char == '*' || _char == '!') {
                inside_c_comment_count += 1;
            } else if last_3_chars[1] == '*' && last_3_chars[2] == '/' {
                inside_c_comment_count = inside_c_comment_count.saturating_sub(1);
            }

            if self.config.preprocessor_if_creates_nesting
                && self.config.indent_preprocessor
                && !inside_comment
            {
                if last_3_chars[1] == '#' && last_3_chars[2] == 'i' && _char == 'f' {
                    preproc_add_nesting_on_next_line = true;
                } else if (last_3_chars[0] == '#' && last_3_chars[1] == 'e')
                    && ((last_3_chars[2] == 'n' && _char == 'd')
                        || (last_3_chars[2] == 'l' && _char == 'i')
                        || (last_3_chars[2] == 'l' && _char == 's'))
                {
                    // Remove everything until the beginning of the line.
                    let mut chars_to_remove = last_3_chars.len(); // skip already added chars
                    for check in output.chars().rev().skip(last_3_chars.len()) {
                        if check != ' ' && check != '\t' {
                            break;
                        }
                        chars_to_remove += 1;
                    }
                    for _ in 0..chars_to_remove {
                        output.pop();
                    }

                    // Decrease nesting.
                    nesting_count = nesting_count.saturating_sub(1);

                    // Add new nesting.
                    output += &indentation_text.repeat(nesting_count);

                    // Add removed chars.
                    output += &last_3_chars.iter().collect::<String>();

                    if _char == 'i' || _char == 's' {
                        // #elif or #else
                        preproc_add_nesting_on_next_line = true;
                    }
                }
            }

            // Determine if we are inside of a comment.
            if last_3_chars[1] == '/' && last_3_chars[2] == '/' {
                inside_comment = true;
                last_comment_line = String::new();
            }

            // Update last input chars.
            last_3_chars[0] = last_3_chars[1];
            last_3_chars[1] = last_3_chars[2];
            last_3_chars[2] = _char;

            if inside_comment || inside_c_comment_count > 0 {
                // Just copy the char, don't do anything else.
                output.push(_char);
                last_comment_line.push(_char);

                // Check if we don't need to format code.
                if last_comment_line == NOFORMAT_BEGIN_COMMENT {
                    inside_no_format = true;
                } else if last_comment_line == NOFORMAT_END_COMMENT {
                    inside_no_format = false;
                }

                continue;
            } else if inside_no_format {
                // Just copy the char, don't run any additional logic.
                output.push(_char);
                continue;
            }

            if _char != ' ' {
                last_non_space_char_is_backslash = _char == '\\';
            }

            if _char == '{' {
                // Remove everything until text.
                let mut chars_to_remove = 0;
                let mut text_starts_with_backslash = false;
                for check in output.chars().rev() {
                    if check != ' ' && check != '\t' && check != '\n' && check != '\r' {
                        text_starts_with_backslash = check == '\\';
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
                        if prev_line_ended_with_backslash && text_starts_with_backslash {
                            // Most likelly we got here from this code:
                            // #define MACRO \
                            // ...           \
                            // {
                            // and now we have:
                            // #define MACRO \
                            // ...           \{

                            // Remove backslash.
                            output.pop();

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
                        }

                        // Make sure previous line is not a comment otherwise our stuff will be inside of a comment:
                        // struct Foo // comment
                        // {
                        // can become this:
                        // struct Foo // comment {
                        let mut found_comment = false;

                        // Read the previous line.
                        let mut line_before = String::new();
                        for check in output.chars().rev() {
                            if check == '\n' {
                                break;
                            }
                            line_before.push(check);
                        }
                        line_before = line_before.chars().rev().collect();

                        if !Self::is_text_starts_with_comment(&line_before) {
                            // See if it has a comment.
                            let line_before_chars_count = line_before.chars().count();
                            let mut skipped_chars_count: usize = 0;
                            let mut copy_chars = false;
                            let mut last_empty_chars_count: usize = 0;
                            let mut line_before_iter = line_before.chars().peekable();
                            while let Some(prev_line_char) = line_before_iter.next() {
                                if copy_chars {
                                    output.push(prev_line_char);
                                }

                                if prev_line_char == '/' {
                                    if let Some('/') = line_before_iter.peek() {
                                        found_comment = true;

                                        // Remove everything until this comment.
                                        let mut additional_chars_to_remove =
                                            line_before_chars_count - skipped_chars_count;
                                        #[cfg(windows)]
                                        {
                                            // also consider `\r`
                                            additional_chars_to_remove =
                                                additional_chars_to_remove.saturating_sub(1);
                                        }
                                        chars_to_remove += additional_chars_to_remove;

                                        // Also remove empty text before this comment.
                                        chars_to_remove += last_empty_chars_count.saturating_sub(1);

                                        for _ in 0..chars_to_remove {
                                            output.pop();
                                        }

                                        // Add a brace.
                                        output.push(' ');
                                        output.push('{');
                                        output.push(' ');

                                        // Now copy everything until end of line.
                                        output.push('/');
                                        copy_chars = true;
                                    }
                                }

                                if prev_line_char == ' ' || prev_line_char == '\t' {
                                    last_empty_chars_count += 1;
                                } else {
                                    last_empty_chars_count = 0;
                                }

                                skipped_chars_count += 1;
                            }

                            if !found_comment {
                                // Add a space and a brace.
                                output.push(' ');
                                output.push(_char);
                            }
                        } else {
                            // Just put bracket to a new line.
                            output += LINE_ENDING;
                            output += &indentation_text.repeat(nesting_count);
                            output.push(_char);
                        }

                        // Increase nesting.
                        nesting_count += 1;

                        // Before inserting a new line check if we are inside of a multi-line macro.
                        if !prev_line_ended_with_backslash {
                            // Insert a new line.
                            is_on_new_line = true;
                            output += LINE_ENDING;
                            output += &indentation_text.repeat(nesting_count);
                            consecutive_empty_new_line_count += 1;
                        }
                    }
                    NewLineAroundOpenBraceRule::Before => {
                        // Before inserting a new line check if we are inside of a multi-line macro.
                        if prev_line_ended_with_backslash {
                            if let Some(last_char) = output.chars().rev().next() {
                                if last_char != '\\' {
                                    output.push('\\');
                                }
                            }
                        }

                        // Insert a new line.
                        is_on_new_line = true;
                        output += LINE_ENDING;
                        output += &indentation_text.repeat(nesting_count);

                        // Add brace.
                        output.push(_char);

                        if prev_line_ended_with_backslash {
                            output.push('\\');
                        }

                        // Add new line with increased nesting.
                        nesting_count += 1;
                        output += LINE_ENDING;
                        output += &indentation_text.repeat(nesting_count);
                        consecutive_empty_new_line_count += 1;
                    }
                }

                // Ignore everything until we find some text.
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

                // Don't add a new line if this line was started with `#`
                // we likelly need to keep the code on the same line.
                if !line_started_with_preprocessor {
                    // Add a new line.
                    output += LINE_ENDING;
                    output += &indentation_text.repeat(nesting_count);
                } else {
                    output.push(' '); // just add a space after text
                }

                // Copy brace.
                output.push(_char);

                // struct Foo{
                // Don't insert a new line here, here is an example why:
                // };
                // The `;` will be on the new line if we insert one.
            } else if _char == '[' || _char == '(' {
                output.push(_char);

                // Add space if needed.
                if self.config.spaces_in_brackets {
                    output.push(' ');
                }

                // Wait for text or an end of line.
                ignore_until_text = true;
                stop_ignoring_if_end_of_line = true;

                // Increase nesting if will be on new line (while inside braces).
                nesting_count += 1;
            } else if _char == ']' || _char == ')' {
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

                // Decrease nesting if will be on new line.
                nesting_count = nesting_count.saturating_sub(1);
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

        if inside_no_format {
            return Err(format!(
                "{} was found but no matching{} detected",
                NOFORMAT_BEGIN_COMMENT, NOFORMAT_END_COMMENT
            ));
        }

        Ok(output)
    }

    /// Checks complex formatting rules that require prior parsing (tokens required).
    fn check_complex_rules(
        &self,
        complex_tokens: Vec<(parser::ComplexToken<'_>, SimpleSpan)>,
    ) -> Result<(), String> {
        // Prepare some variables to determine if we are inside of a global scope or inside of some function.
        let mut is_global_scope = true;
        let mut is_inside_nolint = false;
        let mut scope_nesting_count = 0;

        let mut token_iter = complex_tokens.iter().peekable();
        while let Some((complex_token, _)) = token_iter.next() {
            // Check for nolint section.
            if let Other(Token::Comment(text)) = *complex_token {
                if text.starts_with("NOLINTBEGIN") {
                    is_inside_nolint = true;
                } else if text.starts_with("NOLINTEND") {
                    is_inside_nolint = false;
                }
            }

            // Skip this token if nolint.
            if is_inside_nolint {
                continue;
            }
            // Check if next token is a nolint.
            else if let Some((Other(Token::Comment(text)), _)) = token_iter.peek() {
                if text.starts_with("NOLINT") {
                    continue;
                }
            }

            match complex_token {
                VariableDeclaration(_type, name) => {
                    self.check_variable_name(name, *_type, is_global_scope)?;
                }
                Struct(info) => {
                    is_global_scope = false;

                    // Check docs.
                    if self.config.require_docs_on_structs {
                        Self::check_struct_docs(info)?;
                    }

                    // Check name case.
                    if let Some(case) = self.config.struct_case {
                        Self::check_name_case(info.name, case)?;
                    }

                    // Check fields.
                    for field_info in &info.fields {
                        self.check_variable_name(
                            field_info.name,
                            field_info._type,
                            is_global_scope,
                        )?;
                    }

                    // Check field docs.
                    if self.config.require_docs_on_fields {
                        Self::check_struct_field_docs(info)?;
                    }

                    is_global_scope = true;
                }
                Function(info) => {
                    is_global_scope = false;
                    scope_nesting_count = 0;

                    // Check docs.
                    if self.config.require_docs_on_functions {
                        Self::check_function_docs(info)?;
                    }

                    // Check name case.
                    if let Some(case) = self.config.function_case {
                        Self::check_name_case(info.name, case)?;
                    }

                    // Check args.
                    for info in &info.args {
                        self.check_variable_name(info.name, info._type, is_global_scope)?;
                    }
                }
                Other(token) => {
                    if !is_global_scope {
                        if *token == Token::Ctrl('{') {
                            scope_nesting_count += 1;
                        } else if *token == Token::Ctrl('}') {
                            if scope_nesting_count == 0 {
                                // Unexpected, we probably have something wrong in other place.
                                return Err("found '}' but scope nesting counter is already zero"
                                    .to_owned());
                            } else {
                                scope_nesting_count -= 1;
                                if scope_nesting_count == 0 {
                                    is_global_scope = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_inside_nolint {
            return Err("`NOLINTBEGIN` was found but no matching `NOLINTEND` detected".to_owned());
        }

        Ok(())
    }

    /// Checks various complex formatting rules on the specified variable.
    ///
    /// # Return
    /// `Ok` if the name is correct (according to the rules), otherwise `Err` that container
    /// an error message with suggestions according to the rules.
    fn check_variable_name(
        &self,
        mut name: &str,
        _type: Type,
        is_global_scope: bool,
    ) -> Result<(), String> {
        // Check global variable prefix.
        if let Some(global_prefix) = &self.config.global_variable_prefix {
            // TODO: rework this branch into a single one when Rust's #53667 is resolved
            if is_global_scope {
                if !name.starts_with(global_prefix) {
                    return Err(format!(
                        "\"{}\" has incorrect prefix because it's a global variable, the correct name is probably \"{}\"",
                        name, global_prefix.to_owned() + name
                    ));
                }

                // Make sure the name is in ASCII because we will create a new slice using bytes not chars.
                if !name.is_ascii() && !global_prefix.is_ascii() {
                    return Err(format!(
                        "expected global prefix rule \"{}\" and \"{}\" to have an ASCII-only name",
                        global_prefix, name
                    ));
                }

                // Remove global prefix from further checks.
                name = &name[global_prefix.len()..];
            }
        }

        // Check case.
        if let Some(case) = self.config.variable_case {
            Self::check_name_case(name, case)?
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

    fn check_name_case(name: &str, case: Case) -> Result<(), String> {
        match Self::is_case_different(name, case) {
            Ok(_) => Ok(()),
            Err(correct) => Err(format!(
                "\"{}\" has incorrect case, the correct case is \"{}\"",
                name, correct
            )),
        }
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

    /// Checks that the documentation for the specified function is written for return type and all arguments.
    ///
    /// # Return
    /// `Ok` if docs are correct, otherwise `Err` with a meaningful message about incorrect docs.
    fn check_function_docs(func_info: &FunctionInfo) -> Result<(), String> {
        // Make sure docs are not empty.
        if func_info.docs.is_empty() {
            return Err(format!(
                "expected to find documentation for the function \"{}\"",
                func_info.name
            ));
        }

        // Make sure docs are using ASCII characters since we will use `find` on bytes not chars.
        if !func_info.docs.is_ascii() {
            return Err(format!(
                "expected the documentation for the function \"{}\" to only use ASCII characters",
                func_info.name
            ));
        }

        // Check return docs.
        let return_doc_pos = func_info.docs.find("@return");
        if func_info.return_type != Type::Void {
            if return_doc_pos.is_none() {
                return Err(format!(
                    "expected to find documentation of the return value for the function \"{}\"",
                    func_info.name
                ));
            }
        } else if return_doc_pos.is_some() {
            // Make sure there is no "return" docs (since it's void).
            return Err(format!(
                "found documentation of the VOID return value for the function \"{}\"",
                func_info.name
            ));
        }

        // Collect all args written in the docs.
        let param_keyword = "@param ";
        let mut documented_args: Vec<String> = Vec::new();
        let found_arg_docs: Vec<_> = func_info.docs.match_indices(param_keyword).collect();
        let docs_as_bytes = func_info.docs.as_bytes();
        for (pos, _) in found_arg_docs {
            let mut current_pos = pos + param_keyword.len();
            let mut arg_name = String::new();

            while current_pos < docs_as_bytes.len() {
                let _char = docs_as_bytes[current_pos];
                if _char as char == ' ' {
                    if arg_name.is_empty() {
                        current_pos += 1;
                        continue;
                    } else {
                        break;
                    }
                }

                arg_name += &(_char as char).to_string();
                current_pos += 1;
            }

            documented_args.push(arg_name);
        }

        // Check argument docs.
        for info in &func_info.args {
            if info.is_using_semantic {
                // Don't require docs for arguments with semantics.
                continue;
            }
            if !documented_args.iter().any(|name| name == info.name) {
                return Err(format!(
                    "expected to find documentation for the argument \"{}\" of the function \"{}\"",
                    info.name, func_info.name
                ));
            }
        }

        // Check if there are argument comments that don't reference an actual argument.
        for doc_arg_name in documented_args {
            if !func_info.args.iter().any(|info| info.name == doc_arg_name) {
                return Err(format!(
                    "found documentation for a non-existing argument \"{}\" of the function \"{}\"",
                    doc_arg_name, func_info.name
                ));
            }
        }

        Ok(())
    }

    /// Checks that the documentation for the specified struct is written correctly.
    ///
    /// # Return
    /// `Ok` if docs are correct, otherwise `Err` with a meaningful message about incorrect docs.
    fn check_struct_docs(struct_info: &StructInfo) -> Result<(), String> {
        // Make sure docs are not empty.
        if struct_info.docs.is_empty() {
            return Err(format!(
                "expected to find documentation for the struct \"{}\"",
                struct_info.name
            ));
        }

        Ok(())
    }

    /// Checks if the specified text starts with a comment while ignoring any whitespace
    /// in the beginning.
    ///
    /// # Examples
    /// ```
    /// assert!(is_text_starts_with_comment("   // comment"), true);
    /// assert!(is_text_starts_with_comment("  3// comment"), false);
    /// ```
    fn is_text_starts_with_comment(text: &str) -> bool {
        text.chars()
            .filter(|&_char| _char != ' ' && _char != '\t')
            .collect::<String>()
            .starts_with("//")
    }

    /// Checks that the documentation for fields of the specified struct are written correctly.
    ///
    /// # Return
    /// `Ok` if docs are correct, otherwise `Err` with a meaningful message about incorrect docs.
    fn check_struct_field_docs(struct_info: &StructInfo) -> Result<(), String> {
        for info in &struct_info.fields {
            // Make sure docs are not empty.
            if info.docs.is_empty() {
                return Err(format!(
                    "expected to find documentation for the struct field \"{}\"",
                    info.name
                ));
            }
        }

        Ok(())
    }
}
