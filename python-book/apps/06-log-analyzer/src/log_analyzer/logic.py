"""Log parsing, analysis and reporting."""

import re
from collections import Counter
from collections.abc import Callable, Hashable, Iterable, Iterator

from log_analyzer.evaluator import compile_filter, to_number
from log_analyzer.models import LEVELS, LogEntry, Summary

# Named groups give the match readable keys.
_LINE = re.compile(
    r"(?P<timestamp>\S+) (?P<level>DEBUG|INFO|WARN|ERROR) (?P<service>\S+) (?P<message>.*)"
)
_FIELD = re.compile(r"(\w+)=(\S+)", re.ASCII)


def parse_line(line: str) -> LogEntry | None:
    """Parse one line, or return None if it does not match the format."""
    match = _LINE.fullmatch(line)
    if match is None:
        return None
    message = match["message"]
    fields = dict(_FIELD.findall(message))  # later duplicates win
    return LogEntry(line, match["timestamp"], match["level"], match["service"], message, fields)


def filter_lazily[T](items: Iterable[T], predicate: Callable[[T], bool]) -> Iterator[T]:
    """Lazy generic filter: nothing runs until the consumer pulls values."""
    for item in items:
        if predicate(item):
            yield item


def parse_entries(lines: Iterable[str]) -> tuple[list[LogEntry], int]:
    """Return (entries, skipped_count). Blank lines are ignored, not skipped."""
    entries: list[LogEntry] = []
    skipped = 0
    for line in lines:
        if not line.strip():
            continue
        if (entry := parse_line(line)) is None:
            skipped += 1
        else:
            entries.append(entry)
    return entries, skipped


def analyze(lines: Iterable[str], filter_source: str | None = None) -> Summary:
    """`filter_source=None` matches everything. Raises FilterSyntaxError on a bad filter."""
    matches = (lambda _entry: True) if filter_source is None else compile_filter(filter_source)
    entries, skipped = parse_entries(lines)
    return Summary(len(entries), skipped, list(filter_lazily(entries, matches)))


def count_by[T, K: Hashable](items: Iterable[T], key: Callable[[T], K]) -> dict[K, int]:
    """Count items by key, in first-seen order."""
    return dict(Counter(key(item) for item in items))


def average_ms(entries: Iterable[LogEntry]) -> float | None:
    """Mean of the numeric `ms` fields, or None if there are none."""
    values = [
        number
        for entry in entries
        if (text := entry.fields.get("ms")) is not None and (number := to_number(text)) is not None
    ]
    return sum(values) / len(values) if values else None


def format_report(summary: Summary, *, print_matches: bool = False) -> list[str]:
    """Render the summary as lines."""
    matched = summary.matched
    lines: list[str] = []

    if print_matches:
        lines.extend(entry.raw for entry in matched)
    lines.append(f"matched: {len(matched)} of {summary.total}")

    if matched:
        by_level = count_by(matched, lambda e: e.level)
        levels = " ".join(
            f"{level}={by_level[level]}" for level in reversed(LEVELS) if level in by_level
        )
        lines.append(f"levels: {levels}")

        by_service = sorted(
            count_by(matched, lambda e: e.service).items(),
            key=lambda item: (-item[1], item[0].casefold()),
        )
        lines.append("services: " + " ".join(f"{name}={count}" for name, count in by_service))

        if (avg := average_ms(matched)) is not None:
            text = f"{avg:.2f}".rstrip("0").rstrip(".")
            lines.append(f"avg ms: {text}")

    if summary.skipped > 0:
        lines.append(f"skipped: {summary.skipped}")
    return lines
