# 02 - Number Guessing Game

Control flow: conditionals, loops, early exit, `match`. The game logic is separated from stdin so tests can pass plain lists. Mirrors the [TypeScript version](../../../typescript-book/apps/02-guessing-game/README.md).

```sh
uv run guess                                  # interactive, random secret
GUESS_SEED=1 uv run guess                     # reproducible secret (63)
GUESS_SEED=1 uv run guess --auto              # binary-search bot: "Bot found 63 in 6 attempts"
printf '50\nabc\n25\nq\n' | GUESS_SEED=1 uv run guess
```

Rules: secret in 1..=100, 10 attempts, `q` quits, invalid input does not cost an attempt. Messages: `Too low`, `Too high`, `Correct! You won in N attempt(s)`, `Out of attempts. The number was N`, `Bye`.

## Where to look

| Topic | File |
|-------|------|
| `if` / early `return`, exhaustive `match` on literals | [logic.py](src/guessing_game/logic.py) `judge`, `describe_hint` |
| `for` with `match`, `continue` and `break` | `play_game` |
| `while` loop (binary search) | `solve` |
| Closure with `nonlocal` state (PRNG) | `create_rng` |
| `re.fullmatch`, comparison chaining (`min <= v <= max`) | `parse_input` |
| Class patterns that destructure dataclasses | `case Guess(value):` |

## Patterns

```python
if guess < secret: return "low"           # early return beats nested else
match hint:
    case "low": ...                        # no fallthrough, no `break` needed
for line in lines:                         # any iterable of str: list in tests, sys.stdin in the CLI
    if invalid: continue
    if done: break
while low <= high: ...
config.min <= value <= config.max         # chained comparison
```

## Dialect notes

- Python has `match`, but no `switch` fallthrough and no `do/while`. `for/else` exists (the `else` runs when the loop was not broken) and is occasionally useful.
- `for line in sys.stdin` is lazy and line-buffered, which replaces TypeScript's async readline. Lines keep their trailing `\n`, so parsing strips them.
- Regex gotchas: `\d` matches non-ASCII digits and `$` matches before a trailing newline; use `[0-9]` and `fullmatch`.
- No `++`; use `attempts += 1`.
- Rust: `loop`, labelled breaks and `match` as an expression; this is where Python's statement-based `match` differs most.
