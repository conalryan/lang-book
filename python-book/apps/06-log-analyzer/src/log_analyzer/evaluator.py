"""Tree-walking interpreter for filter expressions."""

import math
import re
from collections.abc import Callable

from log_analyzer.models import LEVELS, And, Compare, CompareOp, Expr, LogEntry, Not, Or
from log_analyzer.parser import parse

# Strict decimal: Python's float() also accepts "inf", "nan" and "1_000".
_DECIMAL = re.compile(r"[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?")


def to_number(text: str) -> float | None:
    """Parse a finite decimal, or None."""
    if not _DECIMAL.fullmatch(text):
        return None
    value = float(text)
    return value if math.isfinite(value) else None


def _field_value(entry: LogEntry, field: str) -> str | None:
    match field:
        case "level" | "service" | "message" | "timestamp":
            return str(getattr(entry, field))
        case _:
            return entry.fields.get(field)


def _to_text(value: str | float) -> str:
    """Render numbers the way the TypeScript reference does: 500.0 -> "500"."""
    if isinstance(value, float):
        return str(int(value)) if value.is_integer() else str(value)
    return value


def _ordering(actual: str, expected: str | float, field: str) -> float | None:
    """Negative / zero / positive, or None when the values cannot be compared."""
    if field == "level":
        wanted = _to_text(expected).upper()
        if actual not in LEVELS or wanted not in LEVELS:
            return None
        return LEVELS.index(actual) - LEVELS.index(wanted)
    if isinstance(expected, float):
        number = to_number(actual)
        if number is None:
            return None
        return (number > expected) - (number < expected)
    return (actual > expected) - (actual < expected)


def _compare(actual: str | None, op: CompareOp, expected: str | float, field: str) -> bool:
    if actual is None:  # a missing field only satisfies "!="
        return op == "!="
    if op == "~":
        return _to_text(expected) in actual

    order = _ordering(actual, expected, field)
    if order is None:
        return op == "!="

    match op:
        case "=":
            return order == 0
        case "!=":
            return order != 0
        case "<":
            return order < 0
        case "<=":
            return order <= 0
        case ">":
            return order > 0
        case ">=":
            return order >= 0
        case "~":  # handled above; kept so the match stays exhaustive
            return False


def evaluate(expr: Expr, entry: LogEntry) -> bool:
    """Pattern match on the node type and recurse."""
    match expr:
        case And(left, right):
            return evaluate(left, entry) and evaluate(right, entry)
        case Or(left, right):
            return evaluate(left, entry) or evaluate(right, entry)
        case Not(operand):
            return not evaluate(operand, entry)
        case Compare(field, op, value):
            return _compare(_field_value(entry, field), op, value, field)


def compile_filter(source: str) -> Callable[[LogEntry], bool]:
    """Parse once, then return a closure that captures the tree and tests entries cheaply."""
    tree = parse(source)
    return lambda entry: evaluate(tree, entry)
