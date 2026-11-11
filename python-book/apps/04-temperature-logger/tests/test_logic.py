import pytest
from temperature_logger.logic import (
    build_log,
    describe_error,
    format_stats,
    parse_reading,
    report,
)
from temperature_logger.models import (
    BadTimestamp,
    BadValue,
    Err,
    FieldCount,
    Ok,
    ParseError,
    Reading,
    TemperatureLog,
    Unit,
    UnknownUnit,
)


def test_reading_normalises_to_celsius() -> None:
    assert Reading("t", 212, Unit.FAHRENHEIT).celsius == pytest.approx(100)
    assert Reading("t", 273.15, Unit.KELVIN).celsius == pytest.approx(0)
    assert Reading("t", 5, Unit.CELSIUS).celsius == 5


def test_unit_is_a_string_enum() -> None:
    assert isinstance(Unit.CELSIUS, str)
    assert Unit.CELSIUS.value == "C"
    assert Unit("K") is Unit.KELVIN


def test_parse_valid_line_with_case_insensitive_unit() -> None:
    result = parse_reading("2024-01-01T08:00 21.5 c")
    assert result == Ok(Reading("2024-01-01T08:00", 21.5, Unit.CELSIUS))


@pytest.mark.parametrize(
    ("line", "error"),
    [
        ("only two", FieldCount(2)),
        ("2024-13-45T99:99 1 C", BadTimestamp("2024-13-45T99:99")),
        ("2024-02-30T10:00 1 C", BadTimestamp("2024-02-30T10:00")),
        ("2024-01-01T24:00 1 C", BadTimestamp("2024-01-01T24:00")),
        ("2024-01-01 1 C", BadTimestamp("2024-01-01")),
        ("2024-01-01T08:00 hot C", BadValue("hot")),
        ("2024-01-01T08:00 inf C", BadValue("inf")),
        ("2024-01-01T08:00 1 X", UnknownUnit("X")),
    ],
)
def test_parse_rejects(line: str, error: ParseError) -> None:
    assert parse_reading(line) == Err(error)


def test_describe_error_covers_every_variant() -> None:
    assert describe_error(FieldCount(1)) == "expected 3 fields, found 1"
    assert describe_error(BadTimestamp("x")) == "invalid timestamp 'x'"
    assert describe_error(BadValue("x")) == "invalid temperature 'x'"
    assert describe_error(UnknownUnit("x")) == "unknown unit 'x'"


def test_empty_log() -> None:
    log = TemperatureLog()
    assert log.size == 0
    assert log.stats() is None
    assert log.latest() is None


def test_odd_and_even_medians() -> None:
    log = (
        TemperatureLog()
        .add(Reading("a", 1, Unit.CELSIUS))
        .add(Reading("b", 9, Unit.CELSIUS))
        .add(Reading("c", 5, Unit.CELSIUS))
    )
    stats = log.stats()
    assert stats is not None
    assert (stats.count, stats.min, stats.max, stats.mean, stats.median) == (3, 1, 9, 5, 5)
    log.add(Reading("d", 7, Unit.CELSIUS))
    stats = log.stats()
    assert stats is not None
    assert stats.median == 6


def test_latest_uses_timestamp_not_insertion_order() -> None:
    log = TemperatureLog().add(Reading("2024-02", 1, Unit.CELSIUS))
    log.add(Reading("2024-01", 2, Unit.CELSIUS))
    latest = log.latest()
    assert latest is not None
    assert latest.timestamp == "2024-02"


def test_build_log_collects_problems_with_line_numbers() -> None:
    log, problems = build_log(["2024-01-01T08:00 1 C\n", "\n", "bad\n"])
    assert log.size == 1
    assert problems == ["line 3: expected 3 fields, found 1"]


def test_empty_report() -> None:
    assert report([]) == ["count: 0"]
    assert format_stats(None, None) == ["count: 0"]


def test_full_report() -> None:
    lines = [
        "2024-01-01T08:00 18.5 C",
        "2024-01-01T12:00 24 C",
        "2024-01-01T16:00 70 F",
        "oops",
        "2024-01-01T20:00 abc C",
        "2024-01-01T21:00 295.15 K",
    ]
    assert report(lines) == [
        "line 4: expected 3 fields, found 1",
        "line 5: invalid temperature 'abc'",
        "count: 4",
        "min: 18.5 C",
        "max: 24 C",
        "mean: 21.4 C",
        "median: 21.56 C",
        "latest: 2024-01-01T21:00 22 C",
    ]
