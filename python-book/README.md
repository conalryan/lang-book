# Python Book

Seven small, runnable apps that walk through Python one concept cluster at a time. Each app mirrors the [TypeScript reference](../typescript-book/README.md): same layout, same commands, same messages, so the code can be diffed across languages.

## Apps

| # | App | Command | New concepts |
|---|-----|---------|--------------|
| 1 | [calculator](./apps/01-calculator) | `calc` | entry point, primitives, variables, operators, parsing, `Literal` types |
| 2 | [guessing-game](./apps/02-guessing-game) | `guess` | `if`/`match`/loops, `break`/`continue`, closures, iterables |
| 3 | [contacts](./apps/03-contacts) | `contacts` | `list`, `dict`, functions, tuples, unpacking, comprehensions |
| 4 | [temperature-logger](./apps/04-temperature-logger) | `templog` | classes, dataclasses, `Enum`, `None`, `Result` vs exceptions |
| 5 | [library](./apps/05-library) | `library-demo` | ABCs, `Protocol`, generics, inheritance vs composition, DI |
| 6 | [log-analyzer](./apps/06-log-analyzer) | `log-analyzer` | `re`, generators, closures, recursive-descent parser, modules |
| 7 | [todo-api](./apps/07-todo-api) | `todo-api` | dependencies, `async`, FastAPI, Pydantic, public API, tests |

## Toolchain

| Concern | Tool |
|---------|------|
| Language version | Python 3.14 (`.python-version`), installed and pinned by uv |
| Package manager + workspace | uv workspace (`apps/*`), single `uv.lock` |
| Build backend | `uv_build` (src layout, one package per app) |
| Tests + coverage | pytest + pytest-cov |
| Format + lint | Ruff (`ruff format`, `ruff check` with isort, pyupgrade, bugbear, docstring rules) |
| Typecheck | mypy `--strict` (with the Pydantic plugin) |
| Task runner | `make` (thin wrapper over `uv run`) |

Build caching: uv caches wheels and resolution, and pytest, Ruff and mypy keep their own incremental caches (`.pytest_cache`, `.ruff_cache`, `.mypy_cache`). Python has no compile step to cache, so there is no Turborepo equivalent here.

## Getting started

```sh
brew install uv          # or see https://docs.astral.sh/uv/
make install             # uv sync: Python 3.14, every app, dev tools

make check               # lint + typecheck + tests with coverage
make test                # tests only
make fmt                 # format and autofix

uv run calc 3 + 4
uv run pytest apps/03-contacts
```

## Layout

```text
apps/NN-name/
  src/<package>/
    models.py     types, dataclasses, enums, protocols (the data)
    logic.py      pure functions and services (the behaviour)
    io.py         the only module that touches stdin/stdout/files/network
    main.py       entry point: wiring only, returns an exit code
  tests/          pytest tests; one per public behaviour
  pyproject.toml  name, scripts (the CLI command), build backend
```

`logic.py` has no I/O, so tests pass plain lists and `write` callbacks. `main.py` and `io.py` are excluded from coverage because they are thin wiring. Apps 5 to 7 add modules where a concept deserves its own file (`demo.py`, `lexer.py`, `parser.py`, `evaluator.py`, `app.py`).

## Cross-language parity

Commands, messages and exit codes match the TypeScript apps exactly, apart from three deliberate differences:

- `calc tour` is Python-specific (the point of that command is language quirks).
- OS error text differs (for example a missing file in app 6).
- Pydantic's validation messages differ from Zod's (the `issues` prefix `field: ` is the same).

Seeded randomness (`GUESS_SEED`) uses the same mulberry32 generator, so `GUESS_SEED=1` gives the same secret everywhere.

## Python dialect notes

- **Types are optional and external.** Annotations do nothing at runtime; mypy enforces them. `--strict` is the equivalent of TypeScript's `strict`.
- **Modern syntax used throughout**: `type X = ...` aliases, `class C[T]` generics (PEP 695), `match` statements, `TypeIs`, `StrEnum`. Annotations are lazily evaluated in 3.14, so forward references need no quotes.
- **Indentation is syntax; there is no `const`.** Constants are a naming convention (`UPPER_CASE`) plus `Final`.
- **Everything is public.** `_name` means "internal", `__all__` defines a package's public API; nothing is enforced.
- **No `null`/`undefined` split.** Only `None`; use `X | None` and narrow with `is None`.
- **Truthiness is broader than JS**: empty containers and `0` are falsy. `or` returns an operand; there is no `??`.
- **Exceptions are the default error mechanism.** Apps 4 and 5 show the `Result` alternative so you can compare with Rust and TypeScript.
- **Sync by default.** `for line in sys.stdin` replaces async iteration; `async` only appears in app 7 where the framework wants it.
- **Decorators** are plain functions that wrap functions. They appear in app 7 (`@app.get(...)`) and in `@dataclass`, `@property`, `@abstractmethod`.

