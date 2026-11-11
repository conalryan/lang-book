use std::cell::Cell;

use crate::evaluator::{compile_filter, evaluate, to_number};
use crate::lexer::tokenize;
use crate::logic::{
    analyze, average_ms, count_by, filter_lazily, format_report, parse_entries, parse_line,
};
use crate::models::{CompareOp, Expr, FilterSyntaxError, LogEntry, TokenKind, Value};
use crate::parser::parse;

const SAMPLE: [&str; 9] = [
    "2024-03-01T10:00:01Z INFO api GET /users status=200 ms=12",
    "2024-03-01T10:00:02Z INFO api GET /users/7 status=200 ms=8",
    "2024-03-01T10:00:03Z WARN db slow query ms=950",
    "2024-03-01T10:00:04Z ERROR api POST /orders status=500 ms=1200",
    "2024-03-01T10:00:05Z DEBUG cache miss key=user:7",
    "this line is garbage",
    "",
    "2024-03-01T10:00:06Z ERROR db connection timeout ms=3000",
    "2024-03-01T10:00:07Z INFO web GET / status=200 ms=30",
];

fn entry(line: &str) -> LogEntry {
    parse_line(line).unwrap_or_else(|| panic!("bad fixture: {line}"))
}

fn entries() -> Vec<LogEntry> {
    SAMPLE.iter().filter_map(|line| parse_line(line)).collect()
}

fn run(filter: &str) -> Vec<String> {
    let predicate = compile_filter(filter).unwrap();
    entries()
        .iter()
        .filter(|e| predicate(e))
        .map(|e| format!("{}:{}", e.service, e.level))
        .collect()
}

#[test]
fn parse_line_extracts_fields_and_pairs() {
    let e = entry(SAMPLE[3]);
    assert_eq!((e.level.as_str(), e.service.as_str()), ("ERROR", "api"));
    assert_eq!(e.message, "POST /orders status=500 ms=1200");
    assert_eq!(e.fields.len(), 2);
    assert_eq!(e.fields["status"], "500");
    assert_eq!(e.fields["ms"], "1200");
}

#[test]
fn parse_line_rejects_garbage() {
    for line in [
        "nope",
        "2024 TRACE api x",
        " INFO api x",
        "ts INFO  x",
        "ts INFO api",
    ] {
        assert!(parse_line(line).is_none(), "{line:?}");
    }
}

#[test]
fn field_extraction_follows_the_regex_semantics() {
    let e = entry("t INFO s a=1 -x=2 b= c=d=e =f g.h=1 key=user:7");
    let mut pairs: Vec<_> = e
        .fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    pairs.sort_unstable();
    assert_eq!(
        pairs,
        [
            ("a", "1"),
            ("c", "d=e"),
            ("h", "1"),
            ("key", "user:7"),
            ("x", "2")
        ]
    );
}

#[test]
fn tokenize_handles_every_token_kind() {
    let tokens = tokenize(r#"status >= 500 AND (msg ~ "a \"b\"" or not x != -1.5)"#).unwrap();
    let kinds: Vec<&str> = tokens
        .iter()
        .map(|t| match t.kind {
            TokenKind::Ident(_) => "ident",
            TokenKind::Str(_) => "string",
            TokenKind::Number(_) => "number",
            TokenKind::Op(_) => "op",
            TokenKind::Keyword(_) => "keyword",
            TokenKind::LParen | TokenKind::RParen => "paren",
            TokenKind::End => "end",
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "ident", "op", "number", "keyword", "paren", "ident", "op", "string", "keyword",
            "keyword", "ident", "op", "number", "paren", "end"
        ]
    );
    assert_eq!(tokens[7].text(), "a \"b\"");
    assert_eq!(tokens[3].text(), "and");
}

#[test]
fn tokenize_rejects_bad_input() {
    for (source, message) in [
        ("status # 5", "unexpected character '#' at position 7"),
        ("msg ~ \"open", "unterminated string at position 6"),
        ("a - b", "unexpected character '-' at position 2"),
    ] {
        assert_eq!(tokenize(source).unwrap_err().to_string(), message);
    }
}

#[test]
fn and_binds_tighter_than_or() {
    let Expr::Or(_, right) = parse("a = 1 or b = 2 and c = 3").unwrap() else {
        panic!("expected an Or at the root");
    };
    assert!(matches!(*right, Expr::And(..)));
}

#[test]
fn parentheses_and_not() {
    let Expr::Not(operand) = parse("not (a = 1 or b = 2)").unwrap() else {
        panic!("expected a Not at the root");
    };
    assert!(matches!(*operand, Expr::Or(..)));
}

#[test]
fn parse_builds_compare_nodes() {
    let compare = |field: &str, op, value| Expr::Compare {
        field: field.into(),
        op,
        value,
    };
    assert_eq!(
        parse("ms > 100"),
        Ok(compare("ms", CompareOp::Gt, Value::Number(100.0)))
    );
    assert_eq!(
        parse(r#"message ~ "x""#),
        Ok(compare(
            "message",
            CompareOp::Contains,
            Value::Text("x".into())
        ))
    );
    assert_eq!(
        parse("level = ERROR"),
        Ok(compare("level", CompareOp::Eq, Value::Text("ERROR".into())))
    );
}

#[test]
fn parse_rejects_bad_filters() {
    let cases = [
        (
            "",
            "expected a field name but found end of input at position 0",
        ),
        (
            "level",
            "expected an operator but found end of input at position 5",
        ),
        (
            "level =",
            "expected a value but found end of input at position 7",
        ),
        (
            "(level = INFO",
            "expected ')' but found end of input at position 13",
        ),
        ("level = INFO INFO", "unexpected 'INFO' at position 13"),
        ("= 1", "expected a field name but found '=' at position 0"),
    ];
    for (source, message) in cases {
        assert_eq!(
            parse(source).unwrap_err().to_string(),
            message,
            "{source:?}"
        );
    }
}

#[test]
fn levels_compare_by_severity() {
    assert_eq!(run("level >= WARN"), ["db:WARN", "api:ERROR", "db:ERROR"]);
    assert_eq!(run("level < info"), ["cache:DEBUG"]);
    assert_eq!(run("level = error"), ["api:ERROR", "db:ERROR"]);
    assert_eq!(run("level = LOUD"), Vec::<String>::new());
}

#[test]
fn numeric_fields_compare_numerically() {
    assert_eq!(run("ms > 100"), ["db:WARN", "api:ERROR", "db:ERROR"]);
    assert_eq!(run("status = 200 and ms < 10"), ["api:INFO"]);
}

#[test]
fn substring_boolean_logic_and_string_compare() {
    assert_eq!(run(r#"message ~ "timeout""#), ["db:ERROR"]);
    assert_eq!(run("message ~ 500"), ["api:ERROR"]);
    assert_eq!(run("service = api and not status = 200"), ["api:ERROR"]);
    assert_eq!(
        run("service = web or service = cache"),
        ["cache:DEBUG", "web:INFO"]
    );
    assert_eq!(
        run("service > cache and service <= db"),
        ["db:WARN", "db:ERROR"]
    );
    assert_eq!(run("service >= web"), ["web:INFO"]);
}

#[test]
fn missing_or_non_numeric_fields_only_match_not_equal() {
    let total = entries().len();
    assert_eq!(run("status > 0").len(), 4);
    assert_eq!(run("status != 200").len(), total - 3);
    assert_eq!(run("key > 1"), Vec::<String>::new());
    assert_eq!(run("key != 1").len(), total);
}

#[test]
fn evaluate_works_directly_on_a_tree() {
    assert!(evaluate(
        &parse("service = api").unwrap(),
        &entry(SAMPLE[0])
    ));
}

#[test]
fn to_number_is_strict() {
    assert_eq!(to_number("1.5e3"), Some(1500.0));
    for text in ["", "inf", "nan", "1_0", "abc", "infinity"] {
        assert_eq!(to_number(text), None, "{text:?}");
    }
}

#[test]
fn parse_entries_counts_skipped_but_ignores_blanks() {
    let (parsed, skipped) = parse_entries(SAMPLE);
    assert_eq!(parsed.len(), 7);
    assert_eq!(skipped, 1);
}

#[test]
fn filter_lazily_does_not_run_until_iterated() {
    let calls = Cell::new(0);
    let lazy = filter_lazily([1, 2, 3], |n| {
        calls.set(calls.get() + 1);
        *n > 1
    });
    assert_eq!(calls.get(), 0);
    assert_eq!(lazy.collect::<Vec<_>>(), [2, 3]);
    assert_eq!(calls.get(), 3);
}

#[test]
fn count_by_keeps_first_seen_order() {
    assert_eq!(
        count_by(&["b", "a", "b"], |s| (*s).to_string()),
        [("b".to_string(), 2), ("a".to_string(), 1)]
    );
}

#[test]
fn average_ms_ignores_entries_without_ms() {
    assert_eq!(
        average_ms(&[entry(SAMPLE[0]), entry(SAMPLE[4])]),
        Some(12.0)
    );
    assert_eq!(average_ms(&[entry(SAMPLE[4])]), None);
}

#[test]
fn report_summarises_everything_by_default() {
    assert_eq!(
        format_report(&analyze(SAMPLE, None).unwrap(), false),
        [
            "matched: 7 of 7",
            "levels: ERROR=2 WARN=1 INFO=3 DEBUG=1",
            "services: api=3 db=2 cache=1 web=1",
            "avg ms: 866.67",
            "skipped: 1",
        ]
    );
}

#[test]
fn report_for_a_filtered_view_can_print_matches() {
    assert_eq!(
        format_report(&analyze(SAMPLE, Some("level >= WARN")).unwrap(), true),
        [
            SAMPLE[2],
            SAMPLE[3],
            SAMPLE[7],
            "matched: 3 of 7",
            "levels: ERROR=2 WARN=1",
            "services: db=2 api=1",
            "avg ms: 1716.67",
            "skipped: 1",
        ]
    );
}

#[test]
fn report_omits_details_when_nothing_matches() {
    assert_eq!(
        format_report(&analyze(SAMPLE, Some("service = nope")).unwrap(), false),
        ["matched: 0 of 7", "skipped: 1"]
    );
}

#[test]
fn analyze_propagates_filter_errors() {
    let error: FilterSyntaxError = analyze(SAMPLE, Some("level >")).unwrap_err();
    assert_eq!(error.position, 7);
}
