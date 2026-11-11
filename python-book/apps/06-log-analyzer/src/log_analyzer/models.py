"""Types: log entries, tokens, the filter syntax tree and errors."""

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Literal

# Ordered by severity; the index doubles as the rank used by `level >= WARN`.
LEVELS: tuple[str, ...] = ("DEBUG", "INFO", "WARN", "ERROR")
type Level = Literal["DEBUG", "INFO", "WARN", "ERROR"]

type CompareOp = Literal["=", "!=", "<", "<=", ">", ">=", "~"]
type TokenType = Literal["ident", "string", "number", "op", "keyword", "paren", "end"]


@dataclass(frozen=True)
class LogEntry:
    """One parsed log line."""

    raw: str
    timestamp: str
    level: str
    service: str
    message: str
    fields: Mapping[str, str]  # `key=value` pairs found in the message, e.g. status=500


@dataclass(frozen=True)
class Token:
    """A lexical token. `pos` is the character offset in the filter source."""

    type: TokenType
    text: str
    pos: int


# Filter syntax tree: a recursive union. Nodes contain other nodes.
@dataclass(frozen=True)
class And:
    """`left and right`."""

    left: Expr
    right: Expr


@dataclass(frozen=True)
class Or:
    """`left or right`."""

    left: Expr
    right: Expr


@dataclass(frozen=True)
class Not:
    """`not operand`."""

    operand: Expr


@dataclass(frozen=True)
class Compare:
    """`field op value`."""

    field: str
    op: CompareOp
    value: str | float


type Expr = And | Or | Not | Compare


class FilterSyntaxError(ValueError):
    """A malformed filter expression; `position` is where it went wrong."""

    def __init__(self, message: str, position: int) -> None:
        super().__init__(f"{message} at position {position}")
        self.position = position


@dataclass(frozen=True)
class Summary:
    """Result of an analysis run."""

    total: int
    skipped: int
    matched: Sequence[LogEntry]
