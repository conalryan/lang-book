# 04 - Temperature Logger

Reads `<timestamp> <value> <unit>` lines from stdin and prints statistics in Celsius. Focus: classes, enums, optionality and error handling.

```sh
pnpm start               # runs against sample.txt
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
| `const` object + union as an enum | [src/models.ts](src/models.ts) `Unit` |
| Class with `readonly` fields, getter, `#private` | `Reading`, `TemperatureLog` |
| `Result<T, E>` as a tagged union | `Result`, `ok`, `err` |
| Tagged union with payloads (like Rust enums) | `ParseError` |
| Exhaustive `switch` with `never` check | [src/logic.ts](src/logic.ts) `describeError` |
| Optionality: `T \| undefined`, `?.`, `??` | `TemperatureLog.stats`, `formatStats` |
| Return a result vs `throw` | `parseReading` vs `buildLog` |

## Patterns

```ts
type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };
if (result.ok) result.value; else result.error;     // narrowing on the discriminant

const unreachable: never = error;                   // compile error if a variant is unhandled
latest?.timestamp ?? "n/a";                         // optional chaining + default
```

## Dialect notes

- `enum` is deliberately avoided (see the root README). The `as const` object gives you runtime values and a string-literal union that serialises naturally.
- There is no `null` safety by default: `strict` turns on `strictNullChecks`, which is what makes `T | undefined` meaningful.
- `catch (e)` gives `unknown`; narrow with `instanceof Error`.
- Rust: `Result` + `?` propagate errors, TypeScript has no `?`-operator, so you check `.ok` explicitly. Python: exceptions are the default; this app shows the alternative.
