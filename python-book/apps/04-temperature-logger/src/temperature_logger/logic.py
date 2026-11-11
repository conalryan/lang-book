"""Parsing and reporting."""

import math
import re
from collections.abc import Iterable
from datetime import datetime
from typing import assert_never

from temperature_logger.models import (
    BadTimestamp,
    BadValue,
    Err,
    FieldCount,
    Ok,
    ParseError,
    Reading,
    Result,
    Stats,
    TemperatureLog,
    Unit,
    UnknownUnit,
)

_TIMESTAMP = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}")
_DECIMAL = re.compile(r"[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?")


def _valid_timestamp(text: str) -> bool:
    if not _TIMESTAMP.fullmatch(text):
        return False
    try:
        datetime.strptime(text, "%Y-%m-%dT%H:%M")
    except ValueError:
        return False
    return True


def parse_reading(line: str) -> Result[Reading, ParseError]:
    """Parse `<timestamp> <value> <unit>`, e.g. `2024-01-01T08:00 21.5 C`."""
    fields = line.split()
    if len(fields) != 3:
        return Err(FieldCount(len(fields)))
    timestamp, value_text, unit_text = fields

    if not _valid_timestamp(timestamp):
        return Err(BadTimestamp(timestamp))
    if not _DECIMAL.fullmatch(value_text):
        return Err(BadValue(value_text))
    value = float(value_text)
    if not math.isfinite(value):
        return Err(BadValue(value_text))
    try:
        unit = Unit(unit_text.upper())
    except ValueError:
        return Err(UnknownUnit(unit_text))
    return Ok(Reading(timestamp, value, unit))


def describe_error(error: ParseError) -> str:
    """Exhaustive `match`: a new variant without a case is flagged by the type checker."""
    match error:
        case FieldCount(found):
            return f"expected 3 fields, found {found}"
        case BadTimestamp(text):
            return f"invalid timestamp '{text}'"
        case BadValue(text):
            return f"invalid temperature '{text}'"
        case UnknownUnit(text):
            return f"unknown unit '{text}'"
        case _:
            assert_never(error)


def build_log(lines: Iterable[str]) -> tuple[TemperatureLog, list[str]]:
    """Build a log; bad lines become messages instead of aborting (Result vs raise)."""
    log = TemperatureLog()
    problems: list[str] = []

    for line_number, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        match parse_reading(line):
            case Ok(reading):
                log.add(reading)
            case Err(error):
                problems.append(f"line {line_number}: {describe_error(error)}")
    return log, problems


def _round2(value: float) -> str:
    if math.isnan(value):
        return "NaN"
    text = f"{value:.2f}".rstrip("0").rstrip(".")
    return "0" if text in ("", "-0") else text


def format_stats(stats: Stats | None, latest: Reading | None) -> list[str]:
    """Format the statistics block."""
    if stats is None:
        return ["count: 0"]
    return [
        f"count: {stats.count}",
        f"min: {_round2(stats.min)} C",
        f"max: {_round2(stats.max)} C",
        f"mean: {_round2(stats.mean)} C",
        f"median: {_round2(stats.median)} C",
        f"latest: {latest.timestamp if latest else 'n/a'} "
        f"{_round2(latest.celsius if latest else math.nan)} C",
    ]


def report(lines: Iterable[str]) -> list[str]:
    """Full report: problems first, then statistics."""
    log, problems = build_log(lines)
    return [*problems, *format_stats(log.stats(), log.latest())]
