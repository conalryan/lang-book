//! Process I/O.

use std::fs;
use std::io;

/// Read a UTF-8 file and split on LF or CRLF.
pub fn read_file_lines(path: &str) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect())
}

/// Write a line to stdout.
pub fn print_line(text: &str) {
    println!("{text}");
}

/// Write `error: <text>` to stderr.
pub fn print_error(text: &str) {
    eprintln!("error: {text}");
}
