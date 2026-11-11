# 04 - Temperature Logger

Reads `<timestamp> <value> <unit>` lines from stdin and prints statistics in Celsius. Focus: classes, enums, optionality and error handling. Mirrors the [TypeScript version](../../../typescript-book/apps/04-temperature-logger/README.md).

```sh
uv run templog < sample.txt
```

```text
line 4: expected 3 fields, found 1
line 5: invalid temperature 'abc'
count: 4
min: 18.5 C
max: 24 C
mean: 21.4 C
median: 21.56 C
latest: 2024-01-01T21:00 22 C
```

Units: `C`, `F`, `K` (case-insensitive). Bad lines are reported with their line number and skipped.

## Where to look

| Topic | File |
|-------|------|
| `StrEnum`: a real enum whose members are strings | [models.py](src/temperature_logger/models.py) `Unit` |
| Class with `__init__`, `@property`, `Self`, `_private` by convention | `TemperatureLog` |
| `@dataclass(frozen=True)` value object with a computed property | `Reading` |
| Generic `Ok[T]` / `Err[E]` and a `Result` alias | `Result`, `Ok`, `Err` |
| Tagged union with payloads (like Rust enums) | `ParseError` |
| Exhaustive `match` with `assert_never` | [logic.py](src/temperature_logger/logic.py) `describe_error`, `Reading.celsius` |
| Optionality: `T | None`, narrowing, walrus | `stats`, `latest`, `format_stats` |
| Return a result vs `raise` | `parse_reading` vs `build_log` |

## Patterns

```python
class Unit(StrEnum):
    CELSIUS = "C"
Unit("c".upper())                        # lookup by value; raises ValueError if unknown

type Result[T, E] = Ok[T] | Err[E]
match parse_reading(line):
    case Ok(reading): ...
    case Err(error): ...                 # destructures the dataclass

def stats(self) -> Stats | None: ...     # callers must check for None
if (best := ...) is None: ...            # walrus: assign and test in one expression
```

## Dialect notes

- Python enums are real classes (iterable, comparable by identity). TypeScript has to fake this with `as const` objects; Rust enums can carry data, which here is a union of small dataclasses.
- `@dataclass` generates `__init__`, `__eq__` and `__repr__`; `frozen=True` adds immutability and hashing. Pydantic (app 7) is the validating cousin.
- There is no `private`: `_name` is a request, `__name` only mangles. Properties (`@property`) are how you expose computed attributes without call syntax.
- `None` is a singleton: compare with `is None`. mypy narrows `X | None` after the check, which is why `stats` and `latest` return it instead of raising.
- Exceptions are idiomatic Python for exceptional input. This app returns `Result` for expected bad lines to show the Rust-style alternative; `float()` and `datetime.strptime` still raise, and the code converts those into `Err`.
