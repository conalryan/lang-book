//! Process I/O.

use std::io::stdin;

/// stdin as an iterator of lines.
pub fn read_lines() -> impl Iterator<Item = String> {
    stdin().lines().map_while(Result::ok)
}

/// Write a line to stdout.
pub fn print_line(text: &str) {
    println!("{text}");
}
