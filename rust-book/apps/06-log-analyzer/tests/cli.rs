use std::process::Command;

const SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/sample.log");

fn run(args: &[&str]) -> (String, String, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_log-analyzer"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.code(),
    )
}

#[test]
fn prints_a_filtered_summary() {
    let (out, _, code) = run(&[SAMPLE, "--filter", "level >= WARN"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out.lines().take(2).collect::<Vec<_>>(),
        ["matched: 3 of 7", "levels: ERROR=2 WARN=1"]
    );
}

#[test]
fn accepts_short_options_and_equals_syntax() {
    assert_eq!(
        run(&[SAMPLE, "-f", "level >= WARN", "-p"]).0,
        run(&[SAMPLE, "--filter=level >= WARN", "--print"]).0
    );
}

#[test]
fn reports_syntax_errors() {
    assert_eq!(
        run(&[SAMPLE, "-f", "level >"]),
        (
            String::new(),
            "error: expected a value but found end of input at position 7\n".into(),
            Some(1)
        )
    );
}

#[test]
fn reports_missing_files_unknown_options_and_usage() {
    assert_eq!(run(&["/no/such/file.log"]).2, Some(1));
    assert!(
        run(&["/no/such/file.log"])
            .1
            .starts_with("error: cannot read '/no/such/file.log'")
    );
    assert_eq!(
        run(&[SAMPLE, "--bogus"]).1,
        "error: unknown option '--bogus'\n"
    );
    assert_eq!(
        run(&[]).1,
        "error: usage: log-analyzer <file> [--filter <expr>] [--print]\n"
    );
}
