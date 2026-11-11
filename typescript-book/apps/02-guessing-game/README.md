# 02 - Number Guessing Game

Control flow: conditionals, loops, early exit, `switch`. The game logic is separated from stdin so it can be tested with plain arrays.

```sh
pnpm start                                  # interactive, random secret
GUESS_SEED=1 pnpm start                     # reproducible secret (63)
GUESS_SEED=1 pnpm start -- --auto           # binary-search bot: "Bot found 63 in 6 attempts"
printf '50\nabc\n25\nq\n' | GUESS_SEED=1 pnpm start
```

Rules: secret in 1..=100, 10 attempts, `q` quits, invalid input does not cost an attempt. Messages: `Too low`, `Too high`, `Correct! You won in N attempt(s)`, `Out of attempts. The number was N`, `Bye`.

## Where to look

| Topic | File |
|-------|------|
| `if` / `else if`, exhaustive `switch`, early `return` | [src/logic.ts](src/logic.ts) `judge`, `describeHint` |
| `for await ... of` with `continue` and `break` | `playGame` |
| `while` loop (binary search) | `solve` |
| Closure holding private state (PRNG) | `createRng` |
| Regex test, ranges, boolean logic | `parseInput` |
| Top-level `await` | [src/main.ts](src/main.ts) |

## Patterns

```ts
if (guess < secret) return "low";           // early return beats nested else
switch (hint) { case "low": ...; }          // add `default: never` to enforce exhaustiveness
for await (const line of lines) { ... }     // works for sync and async iterables
while (low <= high) { ... break; }
```

## Dialect notes

- `switch` falls through unless you `break`/`return`; `noFallthroughCasesInSwitch` makes that an error.
- Truthiness is loose (`0`, `""`, `NaN`, `null`, `undefined` are falsy). Compare explicitly.
- Python has no `switch` (use `match`); Rust's `match` is an expression and must be exhaustive.
- No ternary-free expression `if`: `a ? b : c` is the expression form.
