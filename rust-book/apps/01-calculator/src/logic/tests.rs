use crate::logic::{calculate, convert, execute, format_number, parse_command, parse_number};
use crate::models::{Command, Operator, Unit};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn run(items: &[&str]) -> Result<String, String> {
    parse_command(&args(items))
        .and_then(|command| execute(&command))
        .map_err(|e| e.to_string())
}

#[test]
fn parse_number_accepts_decimals() {
    assert_eq!(parse_number("-2.5"), Ok(-2.5));
    assert_eq!(parse_number(" 1e3 "), Ok(1000.0));
}

#[test]
fn parse_number_rejects_everything_else() {
    for text in [
        "", "  ", "abc", "12px", "Infinity", "inf", "nan", "1_000", "1e999", "0x10",
    ] {
        let error = parse_number(text).unwrap_err();
        assert_eq!(error.to_string(), format!("'{text}' is not a number"));
    }
}

#[test]
fn calculate_covers_every_operator() {
    let cases = [
        (3.0, Operator::Add, 4.0, 7.0),
        (3.0, Operator::Sub, 4.0, -1.0),
        (3.0, Operator::Mul, 4.0, 12.0),
        (9.0, Operator::Div, 2.0, 4.5),
        (-7.0, Operator::Rem, 3.0, -1.0),
        (2.0, Operator::Pow, 10.0, 1024.0),
    ];
    for (left, op, right, expected) in cases {
        assert_eq!(
            calculate(left, op, right),
            Ok(expected),
            "{left} {op} {right}"
        );
    }
}

#[test]
fn calculate_edge_cases() {
    assert!(calculate(-8.0, Operator::Pow, 0.5).unwrap().is_nan());
    assert_eq!(calculate(10.0, Operator::Pow, 1000.0), Ok(f64::INFINITY));
    for op in [Operator::Div, Operator::Rem] {
        assert_eq!(
            calculate(1.0, op, 0.0).unwrap_err().to_string(),
            "division by zero"
        );
    }
}

#[test]
fn convert_between_units() {
    let show = |value, from, to| format_number(convert(value, from, to).unwrap());
    assert_eq!(show(1.0, Unit::Mi, Unit::Km), "1.609344");
    assert_eq!(show(10.0, Unit::Kg, Unit::Lb), "22.046226");
    assert_eq!(show(100.0, Unit::C, Unit::F), "212");
    assert_eq!(show(-40.0, Unit::F, Unit::C), "-40");
}

#[test]
fn convert_rejects_mismatched_dimensions() {
    let error = convert(1.0, Unit::Km, Unit::Kg).unwrap_err();
    assert_eq!(error.to_string(), "cannot convert length to mass");
}

#[test]
fn format_number_hides_noise() {
    let cases = [
        (0.1 + 0.2, "0.3"),
        (7.0, "7"),
        (-0.0, "0"),
        (-1e-9, "0"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
    ];
    for (value, expected) in cases {
        assert_eq!(format_number(value), expected);
    }
}

#[test]
fn symbols_round_trip() {
    for symbol in ["+", "-", "*", "/", "%", "**"] {
        assert_eq!(Operator::from_symbol(symbol).unwrap().to_string(), symbol);
    }
    assert_eq!(Operator::from_symbol("^"), None);
    assert_eq!(Unit::from_symbol("lb").unwrap().to_string(), "lb");
}

#[test]
fn parse_command_builds_typed_commands() {
    assert_eq!(
        parse_command(&args(&["3", "+", "4"])),
        Ok(Command::Calc {
            left: 3.0,
            op: Operator::Add,
            right: 4.0
        })
    );
    assert_eq!(parse_command(&args(&["tour"])), Ok(Command::Tour));
}

#[test]
fn execute_formats_results() {
    assert_eq!(run(&["3", "+", "4"]).unwrap(), "3 + 4 = 7");
    assert_eq!(run(&["convert", "100", "c", "f"]).unwrap(), "100 c = 212 f");
    assert_eq!(run(&["2", "**", "10"]).unwrap(), "2 ** 10 = 1024");
    assert!(run(&["tour"]).unwrap().contains("0.1 + 0.2"));
}

#[test]
fn parse_command_rejects_bad_input() {
    let usage = crate::logic::USAGE.to_string();
    let cases = [
        (vec!["1", "^", "2"], "unknown operator '^'".to_string()),
        (vec!["x", "+", "2"], "'x' is not a number".to_string()),
        (
            vec!["convert", "1", "km", "parsec"],
            "unknown unit 'parsec'".to_string(),
        ),
        (vec!["convert", "1", "km"], usage.clone()),
        (vec![], usage),
    ];
    for (input, message) in cases {
        assert_eq!(run(&input), Err(message), "{input:?}");
    }
}
