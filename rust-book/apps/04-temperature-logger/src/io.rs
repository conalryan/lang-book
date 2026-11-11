//! Process I/O.

use std::io::stdin;

/// Read all of stdin into a vector of lines.
pub fn read_lines() -> Vec<String> {
    stdin().lines().map_while(Result::ok).collect()
}

/// Write a line to stdout.
pub fn print_line(text: &str) {
    println!("{text}");
}
