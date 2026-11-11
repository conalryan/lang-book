use std::process::Command;

fn calc(args: &[&str]) -> (String, String, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_calc"))
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
fn prints_the_result() {
    assert_eq!(
        calc(&["3", "+", "4"]),
        ("3 + 4 = 7\n".into(), String::new(), Some(0))
    );
}

#[test]
fn reports_errors_on_stderr_with_exit_code_1() {
    assert_eq!(
        calc(&["1", "/", "0"]),
        (String::new(), "error: division by zero\n".into(), Some(1))
    );
}
