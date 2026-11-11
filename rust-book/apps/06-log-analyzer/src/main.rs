use std::process::ExitCode;

use log_analyzer::io::{print_error, print_line, read_file_lines};
use log_analyzer::logic::{analyze, format_report};

const USAGE: &str = "usage: log-analyzer <file> [--filter <expr>] [--print]";

struct Options {
    path: String,
    filter: Option<String>,
    print: bool,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut args = args.into_iter();
    let (mut path, mut filter, mut print) = (None, None, false);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-f" | "--filter" => {
                filter = Some(
                    args.next()
                        .ok_or_else(|| format!("option '{arg}' needs a value"))?,
                );
            }
            "-p" | "--print" => print = true,
            other if other.starts_with("--filter=") => {
                filter = Some(other["--filter=".len()..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("unknown option '{other}'"));
            }
            _ => {
                path.get_or_insert(arg);
            }
        }
    }
    Ok(Options {
        path: path.ok_or_else(|| USAGE.to_string())?,
        filter,
        print,
    })
}

fn run(args: impl IntoIterator<Item = String>) -> Result<Vec<String>, String> {
    let options = parse_args(args)?;
    let lines = read_file_lines(&options.path)
        .map_err(|e| format!("cannot read '{}': {e}", options.path))?;
    let summary = analyze(&lines, options.filter.as_deref()).map_err(|e| e.to_string())?;
    Ok(format_report(&summary, options.print))
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(lines) => {
            for line in &lines {
                print_line(line);
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            print_error(&message);
            ExitCode::FAILURE
        }
    }
}
