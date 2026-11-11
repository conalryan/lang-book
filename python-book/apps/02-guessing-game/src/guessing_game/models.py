"""Types for the guessing game."""

from dataclasses import dataclass
from typing import Literal

type Hint = Literal["low", "high", "correct"]


@dataclass(frozen=True)
class GameConfig:
    """Range and attempt limit."""

    min: int = 1
    max: int = 100
    max_attempts: int = 10


DEFAULT_CONFIG = GameConfig()


@dataclass(frozen=True)
class Guess:
    """A valid in-range guess."""

    value: int


@dataclass(frozen=True)
class QuitInput:
    """The player typed `q` or `quit`."""


@dataclass(frozen=True)
class InvalidInput:
    """Anything else."""


type PlayerInput = Guess | QuitInput | InvalidInput


@dataclass(frozen=True)
class Won:
    """The secret was found."""

    attempts: int


@dataclass(frozen=True)
class Lost:
    """Attempts ran out."""

    secret: int


@dataclass(frozen=True)
class Quit:
    """The player left (or input ended) early."""


type GameOutcome = Won | Lost | Quit
