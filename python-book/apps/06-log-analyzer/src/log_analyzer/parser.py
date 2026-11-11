"""Recursive-descent parser.

One method per grammar rule, lowest precedence first::

    or      := and ("or" and)*
    and     := not ("and" not)*
    not     := "not" not | primary
    primary := "(" or ")" | IDENT OP value
    value   := STRING | NUMBER | IDENT
"""

from log_analyzer.lexer import is_compare_op, tokenize
from log_analyzer.models import And, Compare, Expr, FilterSyntaxError, Not, Or, Token


def _describe(token: Token) -> str:
    return "end of input" if token.type == "end" else f"'{token.text}'"


class _Parser:
    def __init__(self, source: str) -> None:
        self._tokens = tokenize(source)
        self._index = 0

    # The lexer always ends with an "end" token, and we never advance past it.
    def _peek(self) -> Token:
        return self._tokens[self._index]

    def _advance(self) -> Token:
        token = self._tokens[self._index]
        self._index += 1
        return token

    def _at_keyword(self, word: str) -> bool:
        token = self._peek()
        return token.type == "keyword" and token.text == word

    def parse(self) -> Expr:
        expr = self._or()
        trailing = self._peek()
        if trailing.type != "end":
            raise FilterSyntaxError(f"unexpected {_describe(trailing)}", trailing.pos)
        return expr

    def _or(self) -> Expr:
        left = self._and()
        while self._at_keyword("or"):
            self._advance()
            left = Or(left, self._and())
        return left

    def _and(self) -> Expr:
        left = self._not()
        while self._at_keyword("and"):
            self._advance()
            left = And(left, self._not())
        return left

    def _not(self) -> Expr:
        if self._at_keyword("not"):
            self._advance()
            return Not(self._not())
        return self._primary()

    def _primary(self) -> Expr:
        token = self._advance()

        if token.type == "paren" and token.text == "(":
            inner = self._or()
            close = self._advance()
            if close.type != "paren" or close.text != ")":
                raise FilterSyntaxError(f"expected ')' but found {_describe(close)}", close.pos)
            return inner

        if token.type != "ident":
            raise FilterSyntaxError(
                f"expected a field name but found {_describe(token)}", token.pos
            )

        op = self._advance()
        if op.type != "op" or not is_compare_op(op.text):
            raise FilterSyntaxError(f"expected an operator but found {_describe(op)}", op.pos)

        value = self._advance()
        match value.type:
            case "number":
                return Compare(token.text, op.text, float(value.text))
            case "string" | "ident":
                return Compare(token.text, op.text, value.text)
            case _:
                raise FilterSyntaxError(f"expected a value but found {_describe(value)}", value.pos)


def parse(source: str) -> Expr:
    """Parse a filter expression into a syntax tree."""
    return _Parser(source).parse()
