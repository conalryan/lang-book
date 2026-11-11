"""Process I/O."""

import re
import sys
from pathlib import Path


def read_file_lines(path: str) -> list[str]:
    """Read a UTF-8 file and split on LF or CRLF."""
    return re.split(r"\r?\n", Path(path).read_text(encoding="utf-8"))


def print_line(text: str) -> None:
    """Write a line to stdout."""
    print(text)


def print_error(text: str) -> None:
    """Write `error: <text>` to stderr."""
    print(f"error: {text}", file=sys.stderr)
