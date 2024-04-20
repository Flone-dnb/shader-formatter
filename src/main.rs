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

    // Read file.
    let file_content = match std::fs::read_to_string(path.clone()) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to read the file, error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Format code.
    let formatter = Formatter::new(config);
    let output = match formatter.format(&file_content) {
        Ok(o) => o,
        Err(msg) => {
            println!("{}", msg);
            return ExitCode::FAILURE;
        }
    };

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

    ExitCode::SUCCESS
}
