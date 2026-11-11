# 02 - Number Guessing Game

Control flow: conditionals, loops, early exit, `match`. The game logic is separated from stdin so tests pass plain arrays. Mirrors the [TypeScript](../../../typescript-book/apps/02-guessing-game/README.md) and [Python](../../../python-book/apps/02-guessing-game/README.md) versions.

```sh
cargo run -q --bin guess                          # interactive, random secret
GUESS_SEED=1 cargo run -q --bin guess             # reproducible secret (63)
GUESS_SEED=1 cargo run -q --bin guess -- --auto   # bot: "Bot found 63 in 6 attempts"
printf '50\nabc\n25\nq\n' | GUESS_SEED=1 cargo run -q --bin guess
```

Rules: secret in 1..=100, 10 attempts, `q` quits, invalid input does not cost an attempt. Messages: `Too low`, `Too high`, `Correct! You won in N attempt(s)`, `Out of attempts. The number was N`, `Bye`.

## Where to look

| Topic | File |
|-------|------|
| `match` on `Ordering`, exhaustive enum `match` | [logic.rs](src/logic.rs) `judge`, `describe_hint` |
| `for` with `match`, `continue`, `break` | `play_game` |
| `while` loop (binary search) | `solve` |
| Closure that owns and mutates state (`impl FnMut`, `move`) | `create_rng` |
| Wrapping integer arithmetic | `create_rng` |
| Generic over any iterable of string-likes | `play_game<I>` |
| Range check with `RangeInclusive::contains` | `parse_input` |

## Patterns

```rust
match guess.cmp(&secret) {
    Ordering::Less => Hint::Low,
    Ordering::Greater => Hint::High,
    Ordering::Equal => Hint::Correct,
}

for line in lines {              // anything `IntoIterator` with `AsRef<str>` items
    match parse_input(line.as_ref(), config) {
        PlayerInput::Invalid => { write(..); continue; }
        PlayerInput::Guess(n) => { ... if done { break; } }
        PlayerInput::Quit => return GameOutcome::Quit,
    }
}

pub fn create_rng(seed: u32) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || { state = state.wrapping_add(0x6D2B_79F5); ... }   // closure owns `state`
}
```

## Dialect notes

- `if` and `match` are expressions: they produce values, which is why `judge` is one `match`. Conditions must be `bool` (no truthiness).
- Loops: `loop` (infinite), `while`, `for x in iterator`. `break` can carry a value out of `loop`, and labels (`'outer:`) target nested loops.
- Arithmetic overflow panics in debug builds, so the PRNG uses `wrapping_add` / `wrapping_mul` to get the same 32-bit results as JavaScript's `Math.imul` and Python's masking.
- `as u32` on a float truncates toward zero, the same as `Math.floor` for positive values.
- Closures capture by reference unless `move`; a returned closure must `move` its state.
