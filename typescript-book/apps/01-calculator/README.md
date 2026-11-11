# 01 - Calculator

CLI calculator and unit converter. Goal: learn the toolchain and the basic syntax before anything fancy.

```sh
pnpm start 3 + 4                 # 3 + 4 = 7
pnpm start 2 '**' 10             # 2 ** 10 = 1024
pnpm start convert 100 c f       # 100 c = 212 f
pnpm start convert 1 mi km       # 1 mi = 1.609344 km
pnpm start tour                  # annotated tour of primitive types
pnpm start 1 / 0                 # error: division by zero (exit code 1)
```

Operators: `+ - * / % **`. Units: `km mi kg lb c f`. Output numbers are rounded to 6 decimals.

## Where to look

| Topic | File |
|-------|------|
| Entry point, top-level `try/catch`, exit code | [src/main.ts](src/main.ts) |
| `as const` arrays, literal unions, `Record`, discriminated union | [src/models.ts](src/models.ts) |
| `switch`, type guards, parsing, string formatting | [src/logic.ts](src/logic.ts) |
| Primitive types, `const`/`let`, operators, quirks | [src/tour.ts](src/tour.ts) |
| `process.stdout` wrappers | [src/io.ts](src/io.ts) |

## Patterns

```ts
const answer = 42;                    // const: cannot be reassigned
let counter = 0; counter += 1;        // let: can be

type Operator = "+" | "-";            // union of string literals
const OPERATORS = ["+", "-"] as const;
type Op = (typeof OPERATORS)[number]; // derive the union from a value

function isOperator(s: string): s is Operator { ... }   // type guard
const [first, second, ...rest] = args;                  // array destructuring
`${left} ${op} ${right}`;                               // template literal
```

## Dialect notes

- Only one numeric type, `number` (64-bit float). `bigint` exists but never mixes with `number`. No integer division operator: use `Math.trunc(a / b)`.
- `Number("")` is `0` and `Number("12px")` is `NaN`; always validate parsed numbers.
- Use `===`. `typeof null` is `"object"`. Prefer `??` over `||` for defaults.
- Python: `//` and `int` do not exist here. Rust: no overflow panics, silent precision loss instead.
