# 01 - Calculator

CLI calculator and unit converter. Goal: learn the toolchain and the basic syntax before anything fancy. Mirrors the [TypeScript version](../../../typescript-book/apps/01-calculator/README.md).

```sh
uv run calc 3 + 4                 # 3 + 4 = 7
uv run calc 2 '**' 10             # 2 ** 10 = 1024
uv run calc convert 100 c f       # 100 c = 212 f
uv run calc convert 1 mi km       # 1 mi = 1.609344 km
uv run calc tour                  # annotated tour of Python's primitive types
uv run calc 1 / 0                 # error: division by zero (exit code 1)
```

Operators: `+ - * / % **`. Units: `km mi kg lb c f`. Output numbers are rounded to 6 decimals.

## Where to look

| Topic | File |
|-------|------|
| Entry point, `try/except`, exit code, `if __name__ == "__main__"` | [main.py](src/calculator/main.py) |
| `type` aliases, `Literal`, `@dataclass(frozen=True)`, union of dataclasses | [models.py](src/calculator/models.py) |
| `match` (value and class patterns), `TypeIs` guards, strict parsing | [logic.py](src/calculator/logic.py) |
| Primitive types, assignment, operators, quirks | [tour.py](src/calculator/tour.py) |
| `print` wrappers | [io.py](src/calculator/io.py) |

## Patterns

```python
answer = 42                        # no const/let; rebindable names, UPPER_CASE by convention
type Operator = Literal["+", "-"]  # alias for a union of literals (3.12+)
def is_operator(s: str) -> TypeIs[Operator]: ...    # type guard

match args:                        # structural pattern matching
    case ["convert", value, src, dst]: ...
    case [left, op, right]: ...
    case _: raise CalculatorError(USAGE)

f"{left} {op} {right}"            # f-string
a, b, *rest = items                # unpacking
```

## Dialect notes

- `int` is arbitrary precision, `float` is a 64-bit double. `/` is always true division; `//` floors; `%` takes the sign of the divisor. This app uses `math.fmod` so `-7 % 3` is `-1` like TypeScript and Rust.
- `float("inf")`, `float("nan")` and `float("1_000")` all parse. The calculator validates with a regex first, because "is this a number" is a decision you make explicitly.
- `**` on floats can return a `complex` (`(-8) ** 0.5`); `math.pow` raises instead, which the code maps to `NaN`.
- `bool` is a subclass of `int` (`True + True == 2`). Use `is` only for `None`.
- TypeScript: no separate `bigint`; Rust: integer overflow would panic in debug builds, Python simply grows.
