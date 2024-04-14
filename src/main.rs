use chumsky::prelude::*;
use config::Config;
use formatter::Formatter;
use std::io::Write;
use std::{fs::File, process::ExitCode};

mod config;
mod formatter;
mod helpers;
mod parser;
mod rules;
mod tests;

fn main() -> ExitCode {
    // Make sure a path is specified.
    if std::env::args().len() <= 1 {
        println!("expected a path to be specified");
        return ExitCode::FAILURE;
    }

    // Get path.
    let Some(path) = std::env::args().nth(1) else {
        println!("expected a path to be specified");
        return ExitCode::FAILURE;
    };

    // Make sure it's a file.
    let path = std::path::PathBuf::from(path);
    if !path.is_file() {
        println!("expected \"{}\" to point to a file", path.to_string_lossy());
        return ExitCode::FAILURE;
    }

    // Load config.
    let config = match Config::get() {
        Ok(f) => f,
        Err(msg) => {
            println!("{}", msg);
            return ExitCode::FAILURE;
        }
    };

    // Create a formatter.
    let formatter = Formatter::new();

    // Read file.
    let file_content = match std::fs::read_to_string(path.clone()) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to read the file, error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Apply rules that don't need tokens.
    let output = formatter.apply_simple_rules(&config, &file_content);

    // Check if we need to do any token parsing for complex rules.
    if config.local_variable_case.is_some() {
        // Parse tokens.
        let (tokens, errors) = parser::token_parser()
            .parse(output.as_str())
            .into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            for error in errors {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, output.as_str());
                let reason = error.reason();
                println!(
                    "token parser error at line {} column {}, reason: {}",
                    line, column, reason
                );
            }
            return ExitCode::FAILURE;
        }

        // Exit of no tokens returned (not an error).
        if tokens.is_none() {
            println!("token parser returned 0 tokens");
            return ExitCode::SUCCESS;
        }
        let tokens: Vec<(parser::Token<'_>, SimpleSpan)> = tokens.unwrap();

        // Parse statements.
        let (statements, errors) = parser::statement_parser()
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            for error in errors {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, output.as_str());
                let reason = error.reason();
                println!(
                    "statement parser error at line {} column {}, reason: {}",
                    line, column, reason
                );
            }
            return ExitCode::FAILURE;
        }

        match statements {
            None => {} // nothing to do here
            Some(statements) => match formatter.check_complex_rules(&config, statements) {
                Ok(_) => {}
                Err(msg) => {
                    println!("changes required: {}", msg);
                    return ExitCode::FAILURE;
                }
            },
        }
    }

    // Write result to the file.
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(error) => {
            println!("failed to open the file for writing, error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    match write!(file, "{}", output) {
        Ok(_) => {}
        Err(error) => {
            println!("failed to write to the file, error: {}", error);
            return ExitCode::FAILURE;
        }
    }

    0.into()
}
