# 04 - Temperature Logger

Reads `<timestamp> <value> <unit>` lines from stdin and prints statistics in Celsius. Focus: structs, enums with data, `Option`, `Result`. Mirrors the [TypeScript](../../../typescript-book/apps/04-temperature-logger/README.md) and [Python](../../../python-book/apps/04-temperature-logger/README.md) versions.

```sh
cargo run -q --bin templog < sample.txt
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

Units: `C`, `F`, `K` (case-insensitive). Timestamps are `YYYY-MM-DDTHH:MM` and must be real calendar times (no Feb 30, no 24:00). Bad lines are reported with their line number and skipped.

## Where to look

| Topic | File |
|-------|------|
| Plain enum, associated function `Unit::parse` | [models.rs](src/models.rs) |
| Enum whose variants carry different data | `ParseError` |
| `Display` + `std::error::Error` for a real error type | `ParseError` |
| Struct, constructor convention (`Reading::new`), methods | `Reading` |
| Private field and chainable `&mut Self` methods | `TemperatureLog` |
| `Option<&Reading>` (a borrow into the log) | `TemperatureLog::latest` |
| `Result` and the `?` operator | [logic.rs](src/logic.rs) `parse_reading` |
| `let ... else` with a slice pattern | `parse_reading` |
| Collect problems instead of failing | `build_log` |

## Patterns

```rust
pub enum ParseError {
    FieldCount { found: usize },        // struct-like variant
    BadTimestamp(String),               // tuple-like variant
}

let [timestamp, value_text, unit_text] = fields.as_slice() else {
    return Err(ParseError::FieldCount { found: fields.len() });
};
let timestamp = parse_timestamp(timestamp)?;     // early return on Err
let unit = Unit::parse(unit_text).ok_or_else(|| ParseError::UnknownUnit(..))?;

log.add(a).add(b);                      // `add` returns &mut Self
fn stats(&self) -> Option<Stats>        // callers must handle "no data"
```

## Dialect notes

- This is the app where Rust enums matter most. TypeScript needs a discriminated union, Python a union of dataclasses; Rust has it built in, and `match` is checked for exhaustiveness at compile time.
- `Result` + `?` replaces try/catch. `?` also converts the error type via `From`, which is how larger programs combine error types (crates such as `thiserror` automate this).
- Fields are private unless marked `pub`: `TemperatureLog` exposes behaviour, not its `Vec`.
- `max_by` returns the *last* maximum, which is exactly the "ties go to the later reading" rule.
- `f64` is not `Ord` (NaN), so sorting uses `sort_by(f64::total_cmp)`.
- Rust has no struct inheritance and no constructors: `new` is only a convention.
