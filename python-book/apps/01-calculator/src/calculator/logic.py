"""Pure behaviour: parsing, arithmetic, conversion and formatting."""

import math
import re
from typing import TypeIs

from calculator.models import (
    OPERATORS,
    UNIT_TABLE,
    Calc,
    CalculatorError,
    Command,
    Convert,
    Operator,
    Tour,
    Unit,
)
from calculator.tour import tour

USAGE = "usage: calc <number> <op> <number> | calc convert <value> <from> <to> | calc tour"

# Strict decimal: Python's float() also accepts "inf", "nan" and "1_000", which we do not want.
_DECIMAL = re.compile(r"[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?", re.ASCII)


def parse_number(text: str) -> float:
    """Parse a finite decimal number or raise `CalculatorError`."""
    stripped = text.strip()
    if not _DECIMAL.fullmatch(stripped):
        raise CalculatorError(f"'{text}' is not a number")
    value = float(stripped)
    if not math.isfinite(value):
        raise CalculatorError(f"'{text}' is not a number")
    return value


def is_operator(text: str) -> TypeIs[Operator]:
    """Type guard: narrows `str` to `Operator` when it returns True."""
    return text in OPERATORS


def is_unit(text: str) -> TypeIs[Unit]:
    """Type guard for unit names."""
    return text in UNIT_TABLE


def calculate(left: float, op: Operator, right: float) -> float:
    """Apply `op`. `%` uses `math.fmod` (sign of the left operand), not Python's `%`."""
    match op:
        case "+":
            return left + right
        case "-":
            return left - right
        case "*":
            return left * right
        case "/" | "%":
            if right == 0:
                raise CalculatorError("division by zero")
            return left / right if op == "/" else math.fmod(left, right)
        case "**":
            try:
                return math.pow(left, right)
            except OverflowError:
                return math.inf
            except ValueError:  # e.g. (-8) ** 0.5; plain `**` would return a complex number
                return math.nan


def convert(value: float, source: Unit, target: Unit) -> float:
    """Convert between two units of the same dimension via the base unit."""
    src, dst = UNIT_TABLE[source], UNIT_TABLE[target]
    if src.dimension != dst.dimension:
        raise CalculatorError(f"cannot convert {src.dimension} to {dst.dimension}")
    return dst.from_base(src.to_base(value))


def format_number(value: float) -> str:
    """Round to 6 decimals and drop trailing zeros: 0.1 + 0.2 prints as 0.3."""
    if math.isnan(value):
        return "NaN"
    if math.isinf(value):
        return "Infinity" if value > 0 else "-Infinity"
    text = f"{value:.6f}".rstrip("0").rstrip(".")
    return "0" if text in ("", "-0") else text


def parse_command(args: list[str]) -> Command:
    """Turn raw argv (without the program name) into a typed command."""
    match args:
        case ["tour"]:
            return Tour()
        case ["convert", value, source, target]:
            if not is_unit(source):
                raise CalculatorError(f"unknown unit '{source}'")
            if not is_unit(target):
                raise CalculatorError(f"unknown unit '{target}'")
            return Convert(parse_number(value), source, target)
        case ["convert", *_]:
            raise CalculatorError(USAGE)
        case [left, op, right]:
            if not is_operator(op):
                raise CalculatorError(f"unknown operator '{op}'")
            return Calc(parse_number(left), op, parse_number(right))
        case _:
            raise CalculatorError(USAGE)


def execute(command: Command) -> str:
    """Run a command and return the exact text to print."""
    match command:
        case Calc(left, op, right):
            result = format_number(calculate(left, op, right))
            return f"{format_number(left)} {op} {format_number(right)} = {result}"
        case Convert(value, source, target):
            result = format_number(convert(value, source, target))
            return f"{format_number(value)} {source} = {result} {target}"
        case Tour():
            return "\n".join(tour())
