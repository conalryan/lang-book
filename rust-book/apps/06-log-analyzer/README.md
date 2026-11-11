# 06 - Log Analyzer

Parses log lines, filters them with a small query language and prints a summary. Focus: strings, iterators, closures, pattern matching, modules and a recursive-descent parser, which is the classic Rust ownership workout. Mirrors the [TypeScript](../../../typescript-book/apps/06-log-analyzer/README.md) and [Python](../../../python-book/apps/06-log-analyzer/README.md) versions.

```sh
cargo run -q --bin log-analyzer -- sample.log --filter 'level >= WARN'
cargo run -q --bin log-analyzer -- sample.log                                 # everything
cargo run -q --bin log-analyzer -- sample.log -f 'service = api and not status = 200' --print
cargo run -q --bin log-analyzer -- sample.log -f 'message ~ "timeout" or ms > 1000'
```

```text
matched: 3 of 7
levels: ERROR=2 WARN=1
services: db=2 api=1
avg ms: 1716.67
skipped: 1
```

Log line: `<timestamp> <DEBUG|INFO|WARN|ERROR> <service> <message>`; `key=value` pairs in the message become fields.

Filter grammar (`and` binds tighter than `or`):

```text
or      := and ("or" and)*
and     := not ("and" not)*
not     := "not" not | primary
primary := "(" or ")" | field op value
op      := = != < <= > >= ~        (~ is substring)
```

`level` compares by severity (`level >= WARN`). Missing fields never match, except with `!=`. Syntax errors print `error: <message> at position N`.

## Where to look

| Topic | File |
|-------|------|
| Tagged unions for tokens and the syntax tree, `Box<Expr>` | [models.rs](src/models.rs) |
| `impl Iterator` as a hand-written generator | [lexer.rs](src/lexer.rs) `Lexer` |
| Collecting `Result`s into one `Result<Vec<_>, _>` | `tokenize` |
| Recursive descent, one method per rule | [parser.rs](src/parser.rs) |
| `let ... else`, early `return Err(..)` | `parse_primary` |
| Tree-walking `match` on a borrowed tree | [evaluator.rs](src/evaluator.rs) `evaluate` |
| Returning a closure that owns the tree | `compile_filter` |
| `split_once`, string slices, char scanning | [logic.rs](src/logic.rs) `parse_line`, `extract_fields` |
| Lazy iterator adapters, generic `filter_lazily<T>` | `filter_lazily` |
| Manual argument parsing | [main.rs](src/main.rs) |

## Patterns

```rust
pub enum Expr {
    And(Box<Expr>, Box<Expr>),         // Box: a recursive type needs an indirection
    Not(Box<Expr>),
    Compare { field: String, op: CompareOp, value: Value },
}

pub fn evaluate(expr: &Expr, entry: &LogEntry) -> bool {   // borrows; does not consume
    match expr {
        Expr::And(l, r) => evaluate(l, entry) && evaluate(r, entry),
        Expr::Not(e) => !evaluate(e, entry),
        Expr::Compare { field, op, value } => ...,
    }
}

pub fn compile_filter(src: &str) -> Result<impl Fn(&LogEntry) -> bool + use<>, FilterSyntaxError> {
    let tree = parse(src)?;
    Ok(move |entry: &LogEntry| evaluate(&tree, entry))       // closure owns `tree`
}
```

## Dialect notes

- Rust iterators are lazy and zero-cost: `filter`, `map`, `take` build a pipeline that runs when consumed (`collect`, `for`, `sum`). Implementing `Iterator` for a struct is the equivalent of a generator.
- A recursive enum needs `Box` so its size is finite. Evaluating borrows the tree (`&Expr`); the parser *moves* nodes into new parents (`Box::new(left)`), which is the ownership lesson of this app.
- Strings are UTF-8 bytes. Indexing `s[i]` is not allowed; use `chars()` (this lexer collects a `Vec<char>` so positions match the other ports) or byte-based `split_once` / `find`.
- There is no regex in the standard library, and the `regex` crate is deliberately left out so the first-principles parsing is visible. Apps 1 to 6 use only `std`.
- `match` guards, bindings and nested patterns replace chains of `instanceof` / `isinstance`.
