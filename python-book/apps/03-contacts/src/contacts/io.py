"""Process I/O."""

import sys
from collections.abc import Iterable


def read_lines() -> Iterable[str]:
    """Stdin as an iterable of lines."""
    return sys.stdin


def print_line(text: str) -> None:
    """Write a line to stdout."""
    print(text, flush=True)
