#![deny(warnings)]

use config::Config;
use formatter::Formatter;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs::File, process::ExitCode};

mod config;
mod formatter;
mod helpers;
mod parser;
mod rules;
mod tests;

const PRINT_TOKENS_ARG: &str = "--print-tokens";
const ONLY_SCAN_ARG: &str = "--only-scan";

fn get_files_in_directory(dir: &Path) -> Vec<PathBuf> {
    let dir = Path::new(&dir);

    if !dir.is_dir() {
        panic!()
    }

    let paths = fs::read_dir(dir).unwrap();
    let mut output = Vec::new();

    for path in paths {
        let path = path.unwrap().path();

        if path.is_dir() {
            let mut result = get_files_in_directory(&path);
            output.append(&mut result);
            continue;
        }

        output.push(path);
    }

    output
}

fn main() -> ExitCode {
    // Make sure a path is specified.
    if std::env::args().len() == 1 {
        println!("shader formatter (v{})", env!("CARGO_PKG_VERSION"));
        println!();
        println!("expected a path to be specified\n");
        println!("usage:");
        println!(
            "{} <path to file or a directory> <option>",
            std::env::args().next().unwrap()
        );
        println!("\nwhere <option> is one of the following:");
        println!(
            "\"{}\" - prints parsed tokens (used for debugging)\n\
             \"{}\" - only check if formatting is needed or not, don't change the actual file, \
                returns 0 if no formatting is needed",
            PRINT_TOKENS_ARG, ONLY_SCAN_ARG
        );
        return ExitCode::FAILURE;
    }

    // Get path to shaders.
    let Some(path_to_shader) = std::env::args().nth(1) else {
        println!("expected a path to be specified");
        return ExitCode::FAILURE;
    };

    // See if we need to print tokens.
    let print_tokens = if let Some(additional_option) = std::env::args().nth(2) {
        additional_option == PRINT_TOKENS_ARG
    } else {
        false
    };

    // See if we only need to scan.
    let only_scan = if let Some(additional_option) = std::env::args().nth(2) {
        additional_option == ONLY_SCAN_ARG
    } else {
        false
    };

    let specified_path = std::path::PathBuf::from(path_to_shader);
    let mut shader_paths: Vec<PathBuf> = Vec::new();

    let mut shader_directory = specified_path.clone();
    if specified_path.is_file() {
        shader_paths.push(specified_path.clone());

        // Get directory of this shader file.
        shader_directory = match specified_path.parent() {
            Some(path) => path.to_path_buf(),
            None => {
                println!(
                    "failed to get parent directory for file \"{}\"",
                    specified_path.to_string_lossy()
                );
                return ExitCode::FAILURE;
            }
        };
    } else {
        // Recursively get all shader files from the specified directory.
        shader_paths = get_files_in_directory(&specified_path);
    }

    // Load config.
    let config = match Config::get(&shader_directory) {
        Ok(f) => f,
        Err(msg) => {
            println!("{}", msg);
            return ExitCode::FAILURE;
        }
    };

    for path_to_shader in shader_paths {
        // Read file.
        let file_content = match std::fs::read_to_string(path_to_shader.clone()) {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read the file, error: {}", e);
                return ExitCode::FAILURE;
            }
        };

        // Format code.
        let formatter = Formatter::new(config.clone());
        let output = match formatter.format(&file_content, print_tokens) {
            Ok(o) => o,
            Err(msg) => {
                println!("{}", msg);
                return ExitCode::FAILURE;
            }
        };

        if only_scan {
            let diffs = diff::myers::lines(&file_content, &output);

            let mut formatting_needed = false;
            for diff in &diffs {
                if let diff::Result::Left(_) = diff {
                    formatting_needed = true;
                    break;
                } else if let diff::Result::Right(_) = diff {
                    formatting_needed = true;
                    break;
                }
            }

            if formatting_needed {
                println!("formatting is needed, see diff for before and after formatting:");
                for diff in diffs {
                    match diff {
                        diff::Result::Left(l) => println!("-{}", l),
                        diff::Result::Both(l, _) => println!(" {}", l),
                        diff::Result::Right(r) => println!("+{}", r),
                    }
                }
                return ExitCode::FAILURE;
            }
        } else {
            // Write result to the file.
            let mut file = match File::create(path_to_shader) {
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
        }
    }

    ExitCode::SUCCESS
}
