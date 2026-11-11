import re

import pytest
from log_analyzer.evaluator import compile_filter, evaluate, to_number
from log_analyzer.lexer import tokenize
from log_analyzer.logic import (
    analyze,
    average_ms,
    count_by,
    filter_lazily,
    format_report,
    parse_entries,
    parse_line,
)
from log_analyzer.models import And, Compare, FilterSyntaxError, LogEntry, Not, Or
from log_analyzer.parser import parse

SAMPLE = [
    "2024-03-01T10:00:01Z INFO api GET /users status=200 ms=12",
    "2024-03-01T10:00:02Z INFO api GET /users/7 status=200 ms=8",
    "2024-03-01T10:00:03Z WARN db slow query ms=950",
    "2024-03-01T10:00:04Z ERROR api POST /orders status=500 ms=1200",
    "2024-03-01T10:00:05Z DEBUG cache miss key=user:7",
    "this line is garbage",
    "",
    "2024-03-01T10:00:06Z ERROR db connection timeout ms=3000",
    "2024-03-01T10:00:07Z INFO web GET / status=200 ms=30",
]


def entry(line: str) -> LogEntry:
    parsed = parse_line(line)
    assert parsed is not None, f"bad fixture: {line}"
    return parsed


ENTRIES = [e for line in SAMPLE if (e := parse_line(line)) is not None]


def run(filter_source: str) -> list[str]:
    predicate = compile_filter(filter_source)
    return [f"{e.service}:{e.level}" for e in ENTRIES if predicate(e)]


def test_parse_line_extracts_fields_and_pairs() -> None:
    parsed = entry(SAMPLE[3])
    assert (parsed.level, parsed.service) == ("ERROR", "api")
    assert parsed.message == "POST /orders status=500 ms=1200"
    assert dict(parsed.fields) == {"status": "500", "ms": "1200"}


def test_parse_line_rejects_garbage() -> None:
    assert parse_line("nope") is None
    assert parse_line("2024 TRACE api x") is None


def test_tokenize_handles_every_token_kind() -> None:
    tokens = tokenize('status >= 500 AND (msg ~ "a \\"b\\"" or not x != -1.5)')
    assert [t.type for t in tokens] == [
        "ident", "op", "number", "keyword", "paren", "ident", "op", "string", "keyword",
        "keyword", "ident", "op", "number", "paren", "end",
    ]  # fmt: skip
    assert tokens[7].text == 'a "b"'
    assert tokens[3].text == "and"


@pytest.mark.parametrize(
    ("source", "message"),
    [
        ("status # 5", "unexpected character '#' at position 7"),
        ('msg ~ "open', "unterminated string at position 6"),
    ],
)
def test_tokenize_rejects(source: str, message: str) -> None:
    with pytest.raises(FilterSyntaxError, match=re.escape(message)):
        tokenize(source)


def test_and_binds_tighter_than_or() -> None:
    tree = parse("a = 1 or b = 2 and c = 3")
    assert isinstance(tree, Or)
    assert isinstance(tree.right, And)


def test_parentheses_and_not() -> None:
    tree = parse("not (a = 1 or b = 2)")
    assert isinstance(tree, Not)
    assert isinstance(tree.operand, Or)


def test_parse_builds_compare_nodes() -> None:
    assert parse("ms > 100") == Compare("ms", ">", 100.0)
    assert parse('message ~ "x"') == Compare("message", "~", "x")
    assert parse("level = ERROR") == Compare("level", "=", "ERROR")


@pytest.mark.parametrize(
    ("source", "message"),
    [
        ("", "expected a field name but found end of input"),
        ("level", "expected an operator but found end of input"),
        ("level =", "expected a value but found end of input"),
        ("(level = INFO", "expected ')' but found end of input"),
        ("level = INFO INFO", "unexpected 'INFO'"),
        ("= 1", "expected a field name but found '='"),
    ],
)
def test_parse_rejects(source: str, message: str) -> None:
    with pytest.raises(FilterSyntaxError, match=re.escape(message)):
        parse(source)


def test_levels_compare_by_severity() -> None:
    assert run("level >= WARN") == ["db:WARN", "api:ERROR", "db:ERROR"]
    assert run("level < info") == ["cache:DEBUG"]
    assert run("level = error") == ["api:ERROR", "db:ERROR"]
    assert run("level = LOUD") == []


def test_numeric_fields_compare_numerically() -> None:
    assert run("ms > 100") == ["db:WARN", "api:ERROR", "db:ERROR"]
    assert run("status = 200 and ms < 10") == ["api:INFO"]


def test_substring_boolean_logic_and_string_compare() -> None:
    assert run('message ~ "timeout"') == ["db:ERROR"]
    assert run("message ~ 500") == ["api:ERROR"]
    assert run("service = api and not status = 200") == ["api:ERROR"]
    assert run("service = web or service = cache") == ["cache:DEBUG", "web:INFO"]
    assert run("service > cache and service <= db") == ["db:WARN", "db:ERROR"]
    assert run("service >= web") == ["web:INFO"]


def test_missing_or_non_numeric_fields_only_match_not_equal() -> None:
    assert len(run("status > 0")) == 4
    assert len(run("status != 200")) == len(ENTRIES) - 3
    assert run("key > 1") == []
    assert len(run("key != 1")) == len(ENTRIES)


def test_evaluate_works_directly_on_a_tree() -> None:
    assert evaluate(parse("service = api"), entry(SAMPLE[0])) is True


def test_to_number() -> None:
    assert to_number("1.5e3") == 1500
    assert all(to_number(t) is None for t in ["", "inf", "nan", "1_0", "abc"])


def test_parse_entries_counts_skipped_but_ignores_blanks() -> None:
    entries, skipped = parse_entries(SAMPLE)
    assert len(entries) == 7
    assert skipped == 1


def test_filter_lazily_does_not_run_until_iterated() -> None:
    calls = 0

    def predicate(n: int) -> bool:
        nonlocal calls
        calls += 1
        return n > 1

    lazy = filter_lazily([1, 2, 3], predicate)
    assert calls == 0
    assert list(lazy) == [2, 3]
    assert calls == 3


def test_count_by_keeps_first_seen_order() -> None:
    assert list(count_by(["b", "a", "b"], lambda s: s).items()) == [("b", 2), ("a", 1)]


def test_average_ms_ignores_entries_without_ms() -> None:
    assert average_ms([entry(SAMPLE[0]), entry(SAMPLE[4])]) == 12
    assert average_ms([entry(SAMPLE[4])]) is None


def test_report_summarises_everything_by_default() -> None:
    assert format_report(analyze(SAMPLE)) == [
        "matched: 7 of 7",
        "levels: ERROR=2 WARN=1 INFO=3 DEBUG=1",
        "services: api=3 db=2 cache=1 web=1",
        "avg ms: 866.67",
        "skipped: 1",
    ]


def test_report_for_a_filtered_view_can_print_matches() -> None:
    assert format_report(analyze(SAMPLE, "level >= WARN"), print_matches=True) == [
        SAMPLE[2],
        SAMPLE[3],
        SAMPLE[7],
        "matched: 3 of 7",
        "levels: ERROR=2 WARN=1",
        "services: db=2 api=1",
        "avg ms: 1716.67",
        "skipped: 1",
    ]


def test_report_omits_details_when_nothing_matches() -> None:
    assert format_report(analyze(SAMPLE, "service = nope")) == ["matched: 0 of 7", "skipped: 1"]


def test_analyze_propagates_filter_errors() -> None:
    with pytest.raises(FilterSyntaxError):
        analyze(SAMPLE, "level >")
