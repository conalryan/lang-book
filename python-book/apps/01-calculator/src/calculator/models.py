"""Types and constants: operators, units and the parsed command."""

from collections.abc import Callable
from dataclasses import dataclass
from typing import Literal, get_args

# Literal types are checked statically only; `get_args` recovers the values at runtime.
type Operator = Literal["+", "-", "*", "/", "%", "**"]
OPERATORS: tuple[str, ...] = get_args(Operator.__value__)

type Unit = Literal["km", "mi", "kg", "lb", "c", "f"]
type Dimension = Literal["length", "mass", "temperature"]


@dataclass(frozen=True)
class UnitInfo:
    """A unit knows how to convert to and from a base unit (km, kg, celsius)."""

    dimension: Dimension
    to_base: Callable[[float], float]
    from_base: Callable[[float], float]


UNIT_TABLE: dict[str, UnitInfo] = {
    "km": UnitInfo("length", lambda v: v, lambda v: v),
    "mi": UnitInfo("length", lambda v: v * 1.609344, lambda v: v / 1.609344),
    "kg": UnitInfo("mass", lambda v: v, lambda v: v),
    "lb": UnitInfo("mass", lambda v: v * 0.45359237, lambda v: v / 0.45359237),
    "c": UnitInfo("temperature", lambda v: v, lambda v: v),
    "f": UnitInfo("temperature", lambda v: (v - 32) * 5 / 9, lambda v: v * 9 / 5 + 32),
}


@dataclass(frozen=True)
class Calc:
    """`calc 3 + 4`."""

    left: float
    op: Operator
    right: float


@dataclass(frozen=True)
class Convert:
    """`calc convert 100 c f`."""

    value: float
    source: Unit
    target: Unit


@dataclass(frozen=True)
class Tour:
    """`calc tour`."""


# A union of dataclasses: `match` on the class tells the checker which fields exist.
type Command = Calc | Convert | Tour


class CalculatorError(ValueError):
    """Bad user input; the message is printed verbatim after `error: `."""
