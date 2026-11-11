"""Lexer: turn filter source text into tokens."""

import re
from collections.abc import Iterator
from typing import TypeIs

from log_analyzer.models import CompareOp, FilterSyntaxError, Token

_OPS = ("!=", "<=", ">=", "=", "<", ">", "~")  # two-character operators first
_KEYWORDS = ("and", "or", "not")

_NUMBER = re.compile(r"-?[0-9]+(?:\.[0-9]+)?")
_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_.-]*")


def is_compare_op(text: str) -> TypeIs[CompareOp]:
    """Type guard: narrows `str` to `CompareOp`."""
    return text in _OPS


def _scan(source: str) -> Iterator[Token]:
    """Lazily yield tokens. A generator keeps its position between `yield`s."""
    pos = 0

    while pos < len(source):
        char = source[pos]
        op = next((candidate for candidate in _OPS if source.startswith(candidate, pos)), None)

        if char.isspace():
            pos += 1
        elif char in "()":
            yield Token("paren", char, pos)
            pos += 1
        elif char == '"':
            text: list[str] = []
            cursor = pos + 1
            while cursor < len(source) and source[cursor] != '"':
                if source[cursor] == "\\" and cursor + 1 < len(source):
                    cursor += 1
                text.append(source[cursor])
                cursor += 1
            if cursor >= len(source):
                raise FilterSyntaxError("unterminated string", pos)
            yield Token("string", "".join(text), pos)
            pos = cursor + 1
        elif op is not None:
            yield Token("op", op, pos)
            pos += len(op)
        elif number := _NUMBER.match(source, pos):
            yield Token("number", number[0], pos)
            pos = number.end()
        elif word := _IDENT.match(source, pos):
            lower = word[0].lower()
            yield (
                Token("keyword", lower, pos) if lower in _KEYWORDS else Token("ident", word[0], pos)
            )
            pos = word.end()
        else:
            raise FilterSyntaxError(f"unexpected character '{char}'", pos)

    yield Token("end", "", len(source))


def tokenize(source: str) -> list[Token]:
    """All tokens, ending with an `end` token."""
    return list(_scan(source))
