use std::process::ExitCode;

use calculator::io::{print_error, print_line};
use calculator::logic::{execute, parse_command};

/// Entry point: wiring only. `main` may return `ExitCode` instead of calling `process::exit`.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match parse_command(&args).and_then(|command| execute(&command)) {
        Ok(output) => {
            print_line(&output);
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}
