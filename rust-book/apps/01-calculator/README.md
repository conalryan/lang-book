# 01 - Calculator

CLI calculator and unit converter. Goal: learn the toolchain and the basic syntax before anything fancy. Mirrors the [TypeScript](../../../typescript-book/apps/01-calculator/README.md) and [Python](../../../python-book/apps/01-calculator/README.md) versions.

```sh
cargo run -q --bin calc -- 3 + 4              # 3 + 4 = 7
cargo run -q --bin calc -- 2 '**' 10          # 2 ** 10 = 1024
cargo run -q --bin calc -- convert 100 c f    # 100 c = 212 f
cargo run -q --bin calc -- convert 1 mi km    # 1 mi = 1.609344 km
cargo run -q --bin calc -- tour               # annotated tour of Rust's primitives
cargo run -q --bin calc -- 1 / 0              # error: division by zero (exit code 1)
```

Operators: `+ - * / % **`. Units: `km mi kg lb c f`. Output numbers are rounded to 6 decimals.

## Where to look

| Topic | File |
|-------|------|
| `fn main() -> ExitCode`, wiring | [main.rs](src/main.rs) |
| `enum` + `impl`, `Display`, custom error type | [models.rs](src/models.rs) |
| `match`, slice patterns, `Result`, `?`, `ok_or_else` | [logic.rs](src/logic.rs) |
| Primitives, `let`/`mut`, shadowing, `as` casts, `Option` | [tour.rs](src/tour.rs) |
| Unit tests (private access) | [logic/tests.rs](src/logic/tests.rs) |
| Black-box test of the real binary | [tests/cli.rs](tests/cli.rs) |

## Patterns

```rust
let answer = 42;                 // immutable by default
let mut counter = 0; counter += 1;
let spaces = spaces.len();       // shadowing: new binding, new type

match args.as_slice() {          // slice patterns destructure the argument list
    ["tour"] => Ok(Command::Tour),
    ["convert", value, from, to] => { ... }
    [left, op, right] => { ... }
    _ => Err(error(USAGE)),
}

fn parse_number(text: &str) -> Result<f64, CalcError> { ... }   // errors are values
let value = parse_number(text)?;                                 // `?` returns early on Err
```

## Dialect notes

- Integers have fixed widths (`i32` by default) and never convert implicitly. Overflow panics in debug builds; use `wrapping_*`, `checked_*` or `saturating_*` to choose. `as` casts always succeed (truncate, wrap or saturate).
- `/` on integers truncates toward zero; `%` keeps the dividend's sign (like TypeScript, unlike Python). `rem_euclid` gives the always-positive remainder.
- `"inf".parse::<f64>()` succeeds, so `parse_number` checks the characters first, as the other ports do with a regex.
- `str` is borrowed UTF-8 text and `String` is owned; `len()` counts bytes.
- No `null`: `Option<T>`. No exceptions: `Result<T, E>`. A `match` must cover every case.
- A binary crate can return `ExitCode` from `main`; the logic lives in the library so tests can call it.
