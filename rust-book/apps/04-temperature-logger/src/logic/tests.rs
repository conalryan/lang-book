use crate::logic::{build_log, format_stats, parse_reading, report};
use crate::models::{ParseError, Reading, TemperatureLog, Unit};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn reading_normalises_to_celsius() {
    assert!(close(
        Reading::new("t", 212.0, Unit::Fahrenheit).celsius(),
        100.0
    ));
    assert!(close(
        Reading::new("t", 273.15, Unit::Kelvin).celsius(),
        0.0
    ));
    assert!(close(Reading::new("t", 5.0, Unit::Celsius).celsius(), 5.0));
}

#[test]
fn units_parse_case_insensitively() {
    assert_eq!(Unit::parse("k"), Some(Unit::Kelvin));
    assert_eq!(Unit::parse("X"), None);
}

#[test]
fn parse_valid_line_with_case_insensitive_unit() {
    assert_eq!(
        parse_reading("2024-01-01T08:00 21.5 c"),
        Ok(Reading::new("2024-01-01T08:00", 21.5, Unit::Celsius))
    );
}

#[test]
fn parse_rejects_bad_lines() {
    let cases = [
        ("only two", ParseError::FieldCount { found: 2 }),
        (
            "2024-13-45T99:99 1 C",
            ParseError::BadTimestamp("2024-13-45T99:99".into()),
        ),
        (
            "2024-02-30T10:00 1 C",
            ParseError::BadTimestamp("2024-02-30T10:00".into()),
        ),
        (
            "2023-02-29T10:00 1 C",
            ParseError::BadTimestamp("2023-02-29T10:00".into()),
        ),
        (
            "2024-01-01T24:00 1 C",
            ParseError::BadTimestamp("2024-01-01T24:00".into()),
        ),
        (
            "2024-01-01 1 C",
            ParseError::BadTimestamp("2024-01-01".into()),
        ),
        ("2024-01-01T08:00 hot C", ParseError::BadValue("hot".into())),
        ("2024-01-01T08:00 inf C", ParseError::BadValue("inf".into())),
        ("2024-01-01T08:00 1 X", ParseError::UnknownUnit("X".into())),
    ];
    for (line, error) in cases {
        assert_eq!(parse_reading(line), Err(error), "{line}");
    }
}

#[test]
fn leap_days_are_valid() {
    assert!(parse_reading("2024-02-29T23:59 1 C").is_ok());
}

#[test]
fn errors_describe_themselves() {
    assert_eq!(
        ParseError::FieldCount { found: 1 }.to_string(),
        "expected 3 fields, found 1"
    );
    assert_eq!(
        ParseError::BadTimestamp("x".into()).to_string(),
        "invalid timestamp 'x'"
    );
    assert_eq!(
        ParseError::BadValue("x".into()).to_string(),
        "invalid temperature 'x'"
    );
    assert_eq!(
        ParseError::UnknownUnit("x".into()).to_string(),
        "unknown unit 'x'"
    );
}

#[test]
fn empty_log() {
    let log = TemperatureLog::new();
    assert!(log.is_empty());
    assert!(log.stats().is_none());
    assert!(log.latest().is_none());
}

#[test]
fn odd_and_even_medians() {
    let mut log = TemperatureLog::new();
    log.add(Reading::new("a", 1.0, Unit::Celsius))
        .add(Reading::new("b", 9.0, Unit::Celsius))
        .add(Reading::new("c", 5.0, Unit::Celsius));
    let stats = log.stats().unwrap();
    assert_eq!(
        (stats.count, stats.min, stats.max, stats.mean, stats.median),
        (3, 1.0, 9.0, 5.0, 5.0)
    );
    log.add(Reading::new("d", 7.0, Unit::Celsius));
    assert_eq!(log.stats().unwrap().median, 6.0);
    assert_eq!(log.len(), 4);
}

#[test]
fn latest_uses_timestamp_not_insertion_order() {
    let mut log = TemperatureLog::new();
    log.add(Reading::new("2024-02", 1.0, Unit::Celsius))
        .add(Reading::new("2024-01", 2.0, Unit::Celsius));
    assert_eq!(log.latest().unwrap().timestamp, "2024-02");
}

#[test]
fn build_log_collects_problems_with_line_numbers() {
    let (log, problems) = build_log(["2024-01-01T08:00 1 C", "", "bad"]);
    assert_eq!(log.len(), 1);
    assert_eq!(problems, ["line 3: expected 3 fields, found 1"]);
}

#[test]
fn empty_report() {
    assert_eq!(report(Vec::<String>::new()), ["count: 0"]);
    assert_eq!(format_stats(None, None), ["count: 0"]);
}

#[test]
fn full_report() {
    let lines = [
        "2024-01-01T08:00 18.5 C",
        "2024-01-01T12:00 24 C",
        "2024-01-01T16:00 70 F",
        "oops",
        "2024-01-01T20:00 abc C",
        "2024-01-01T21:00 295.15 K",
    ];
    assert_eq!(
        report(lines),
        [
            "line 4: expected 3 fields, found 1",
            "line 5: invalid temperature 'abc'",
            "count: 4",
            "min: 18.5 C",
            "max: 24 C",
            "mean: 21.4 C",
            "median: 21.56 C",
            "latest: 2024-01-01T21:00 22 C",
        ]
    );
}
