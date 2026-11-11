import math
import re

import pytest
from calculator.logic import (
    calculate,
    convert,
    execute,
    format_number,
    is_operator,
    parse_command,
    parse_number,
)
from calculator.models import CalculatorError
from calculator.tour import tour


def test_parse_number_accepts_decimals() -> None:
    assert parse_number("-2.5") == -2.5
    assert parse_number(" 1e3 ") == 1000


@pytest.mark.parametrize("text", ["", "  ", "abc", "12px", "Infinity", "nan", "1_000", "1e999"])
def test_parse_number_rejects(text: str) -> None:
    with pytest.raises(CalculatorError, match="is not a number"):
        parse_number(text)


@pytest.mark.parametrize(
    ("left", "op", "right", "expected"),
    [
        (3, "+", 4, 7),
        (3, "-", 4, -1),
        (3, "*", 4, 12),
        (9, "/", 2, 4.5),
        (-7, "%", 3, -1),
        (2, "**", 10, 1024),
    ],
)
def test_calculate(left: float, op: str, right: float, expected: float) -> None:
    assert is_operator(op)
    assert calculate(left, op, right) == expected


def test_calculate_edge_cases() -> None:
    assert math.isnan(calculate(-8, "**", 0.5))
    assert calculate(10, "**", 1000) == math.inf
    with pytest.raises(CalculatorError, match="division by zero"):
        calculate(1, "/", 0)
    with pytest.raises(CalculatorError, match="division by zero"):
        calculate(1, "%", 0)


def test_convert() -> None:
    assert format_number(convert(1, "mi", "km")) == "1.609344"
    assert format_number(convert(10, "kg", "lb")) == "22.046226"
    assert format_number(convert(100, "c", "f")) == "212"
    assert format_number(convert(-40, "f", "c")) == "-40"


def test_convert_rejects_mismatched_dimensions() -> None:
    with pytest.raises(CalculatorError, match="cannot convert length to mass"):
        convert(1, "km", "kg")


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        (0.1 + 0.2, "0.3"),
        (7.0, "7"),
        (-0.0, "0"),
        (-1e-9, "0"),
        (math.nan, "NaN"),
        (math.inf, "Infinity"),
        (-math.inf, "-Infinity"),
    ],
)
def test_format_number(value: float, expected: str) -> None:
    assert format_number(value) == expected


def test_is_operator() -> None:
    assert is_operator("**")
    assert not is_operator("^")


@pytest.mark.parametrize(
    ("args", "expected"),
    [
        (["3", "+", "4"], "3 + 4 = 7"),
        (["convert", "100", "c", "f"], "100 c = 212 f"),
        (["2", "**", "10"], "2 ** 10 = 1024"),
    ],
)
def test_execute(args: list[str], expected: str) -> None:
    assert execute(parse_command(args)) == expected


def test_execute_tour() -> None:
    assert "0.1 + 0.2" in execute(parse_command(["tour"]))


@pytest.mark.parametrize(
    ("args", "message"),
    [
        (["1", "^", "2"], "unknown operator '^'"),
        (["x", "+", "2"], "'x' is not a number"),
        (["convert", "1", "km", "parsec"], "unknown unit 'parsec'"),
        (["convert", "1", "km"], "usage:"),
        ([], "usage:"),
    ],
)
def test_parse_command_rejects(args: list[str], message: str) -> None:
    with pytest.raises(CalculatorError, match=re.escape(message)):
        parse_command(args)


def test_tour_documents_python_semantics() -> None:
    output = "\n".join(tour())
    assert "0.30000000000000004" in output
    assert "18446744073709551616" in output
    assert "-7 // 2           -4" in output
