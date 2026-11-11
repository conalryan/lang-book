"""Data model: units, Result, parse errors, readings and the log."""

from dataclasses import dataclass
from enum import StrEnum
from typing import Self, assert_never


class Unit(StrEnum):
    """A real enum. `StrEnum` members are also strings, so `Unit.CELSIUS == "C"`."""

    CELSIUS = "C"
    FAHRENHEIT = "F"
    KELVIN = "K"


@dataclass(frozen=True)
class Ok[T]:
    """Success carrying a value."""

    value: T


@dataclass(frozen=True)
class Err[E]:
    """Failure carrying an error."""

    error: E


# Result as a union of two generic dataclasses; `match` narrows it like a Rust enum.
type Result[T, E] = Ok[T] | Err[E]


@dataclass(frozen=True)
class FieldCount:
    """Wrong number of whitespace-separated fields."""

    found: int


@dataclass(frozen=True)
class BadTimestamp:
    """Timestamp is not `YYYY-MM-DDTHH:MM`."""

    text: str


@dataclass(frozen=True)
class BadValue:
    """Value is not a finite number."""

    text: str


@dataclass(frozen=True)
class UnknownUnit:
    """Unit is not C, F or K."""

    text: str


# Tagged union: each variant carries different data, like a Rust enum.
type ParseError = FieldCount | BadTimestamp | BadValue | UnknownUnit


@dataclass(frozen=True)
class Stats:
    """Summary statistics in Celsius."""

    count: int
    min: float
    max: float
    mean: float
    median: float


@dataclass(frozen=True)
class Reading:
    """An immutable measurement. `value` is stored as entered; `celsius` normalises it."""

    timestamp: str
    value: float
    unit: Unit

    @property
    def celsius(self) -> float:
        """The value converted to Celsius."""
        match self.unit:
            case Unit.CELSIUS:
                return self.value
            case Unit.FAHRENHEIT:
                return (self.value - 32) * 5 / 9
            case Unit.KELVIN:
                return self.value - 273.15
            case _:
                assert_never(self.unit)


class TemperatureLog:
    """Append-only collection of readings with derived statistics."""

    def __init__(self) -> None:
        # A leading underscore means "private by convention"; Python does not enforce it.
        self._readings: list[Reading] = []

    def add(self, reading: Reading) -> Self:
        """Append and return self so calls can be chained."""
        self._readings.append(reading)
        return self

    @property
    def size(self) -> int:
        """Number of readings."""
        return len(self._readings)

    def latest(self) -> Reading | None:
        """Newest by timestamp (ties go to the later insertion); None when empty."""
        best: Reading | None = None
        for reading in self._readings:
            if best is None or reading.timestamp >= best.timestamp:
                best = reading
        return best

    def stats(self) -> Stats | None:
        """None when empty: callers must handle "no data" because the type says so."""
        if not self._readings:
            return None
        values = sorted(r.celsius for r in self._readings)
        count = len(values)
        mid = count // 2
        median = values[mid] if count % 2 == 1 else (values[mid - 1] + values[mid]) / 2
        return Stats(
            count=count,
            min=values[0],
            max=values[-1],
            mean=sum(values) / count,
            median=median,
        )
