# 06 - Log Analyzer

Parses log lines, filters them with a small query language, and prints a summary. Focus: string handling, iterators/generators, closures, pattern matching, modules, and a recursive-descent parser.

```sh
pnpm start                                                  # sample.log, level >= WARN
node src/main.ts sample.log                                 # everything
node src/main.ts sample.log -f 'service = api and not status = 200' --print
node src/main.ts sample.log -f 'message ~ "timeout" or ms > 1000'
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
| Types, tagged unions, custom `Error` | [src/models.ts](src/models.ts) |
| Generator function scanning a string | [src/lexer.ts](src/lexer.ts) |
| Recursive descent with nested closures | [src/parser.ts](src/parser.ts) |
| Tree-walking interpreter via `switch` on node type | [src/evaluator.ts](src/evaluator.ts) |
| Closure returned from `compileFilter` | same file |
| Regex with named groups, `matchAll` | [src/logic.ts](src/logic.ts) `parseLine` |
| Lazy generic generator | `filterLazily<T>` |
| Generic grouping helper | `countBy<T, K>` |
| `node:util` `parseArgs` | [src/main.ts](src/main.ts) |

## Patterns

```ts
function* scan(src: string): Generator<Token> { yield token; }   // lazy sequence
const { timestamp, level } = LINE.exec(line)?.groups ?? {};      // named groups
const pred = compileFilter("level >= WARN");                     // parse once, reuse
entries.filter(pred).map((e) => e.service);                      // lambdas + chaining
```

## Dialect notes

- Generators are lazy; array methods (`map`, `filter`) are eager. `Iterator.prototype.map` helpers exist in newer runtimes.
- Strings are UTF-16: `length` and indexing count code units, not characters.
- Regex objects with the `g` flag are stateful (`lastIndex`); prefer `matchAll`.
- Rust: the lexer/parser is a natural fit for `&str` slices and `Peekable<Chars>`, and ownership of the AST (`Box<Expr>`) is the interesting part. Python: `re` named groups and generators map almost 1:1.
