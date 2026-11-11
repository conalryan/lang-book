use std::io::Write;
use std::process::{Command, Stdio};

fn guess(args: &[&str], stdin: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_guess"))
        .args(args)
        .env("GUESS_SEED", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
}

#[test]
fn auto_mode_is_reproducible() {
    assert_eq!(guess(&["--auto"], ""), "Bot found 63 in 6 attempts\n");
}

#[test]
fn plays_from_stdin() {
    let output = guess(&[], "50\nabc\n63\n");
    assert_eq!(
        output,
        "Guess a number between 1 and 100. You have 10 attempts. Type 'q' to quit.\n\
         Too low\n\
         Please enter a whole number between 1 and 100\n\
         Correct! You won in 2 attempts\n"
    );
}
