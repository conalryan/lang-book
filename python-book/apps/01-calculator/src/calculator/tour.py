"""A guided tour of Python's primitive types and variable semantics."""

import math


def tour() -> list[str]:
    """Return the tour as lines so it can be tested."""
    lines: list[str] = []

    def say(label: str, value: object) -> None:
        lines.append(f"{label:<18}{value}")

    # No const/let: every name is rebindable. UPPER_CASE is only a convention for constants.
    answer = 42
    counter = 0
    counter += 1  # augmented assignment: += -= *= /= //= %= **= and the walrus `:=`

    # int is arbitrary precision; float is a 64-bit double; the two never overflow into each other.
    say("int", answer)
    say("0.1 + 0.2", 0.1 + 0.2)
    say("1_000_000", 1_000_000)  # underscores are purely visual
    say("7 / 2", 7 / 2)  # `/` is always true division
    say("7 // 2", 7 // 2)  # floor division
    say("-7 // 2", -7 // 2)  # floors toward negative infinity, unlike Math.trunc
    say("-7 % 3", -7 % 3)  # result takes the sign of the divisor (JS/Rust: dividend)
    say("math.fmod(-7, 3)", math.fmod(-7, 3))
    say("divmod(7, 2)", divmod(7, 2))  # a tuple
    say("2 ** 64", 2**64)  # no bigint suffix needed
    say("nan == nan", math.nan == math.nan)

    # str is immutable Unicode text; f-strings interpolate.
    name = "Python"
    say("f-string", f"Hello, {name}! counter={counter}")
    say("len", len(name))

    # bool is a subclass of int; None is the single "nothing" value (no null/undefined split).
    say("bool", answer > 10 and not answer < 0)
    say("True + True", True + True)
    say("None", None)
    say("type(None)", type(None).__name__)
    zero = answer - 42  # computed so the linter cannot fold the expression
    nothing = None if answer else 0
    say("0 or 5", zero or 5)  # `or` returns an operand, like ||; there is no ?? operator
    say("None or 5", nothing or 5)

    # `==` compares values, `is` compares identity; use `is` only for None.
    say("1 == 1.0", 1 == 1.0)
    say("'1' == 1", "1" == 1)  # type: ignore[comparison-overlap]

    return lines
