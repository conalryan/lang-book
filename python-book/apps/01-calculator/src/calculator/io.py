"""Thin wrappers over the process streams."""

import sys


def print_line(text: str) -> None:
    """Write a line to stdout."""
    print(text)


def print_error(text: str) -> None:
    """Write `error: <text>` to stderr."""
    print(f"error: {text}", file=sys.stderr)
