//! Thin wrappers over the process streams.

/// Write a line to stdout.
pub fn print_line(text: &str) {
    println!("{text}");
}

/// Write `error: <text>` to stderr.
pub fn print_error(text: &str) {
    eprintln!("error: {text}");
}
