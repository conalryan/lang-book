# 06 - Log Analyzer

Parses log lines, filters them with a small query language, and prints a summary. Focus: strings, generators, closures, pattern matching, modules and a recursive-descent parser. Mirrors the [TypeScript version](../../../typescript-book/apps/06-log-analyzer/README.md).

```sh
uv run log-analyzer sample.log --filter 'level >= WARN'
uv run log-analyzer sample.log                                   # everything
uv run log-analyzer sample.log -f 'service = api and not status = 200' --print
uv run log-analyzer sample.log -f 'message ~ "timeout" or ms > 1000'
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
| Dataclass syntax tree, recursive types, custom exception | [models.py](src/log_analyzer/models.py) |
| Generator function scanning a string, `re.Pattern.match(source, pos)` | [lexer.py](src/log_analyzer/lexer.py) |
| Recursive descent as a small class | [parser.py](src/log_analyzer/parser.py) |
| Tree-walking interpreter via class patterns | [evaluator.py](src/log_analyzer/evaluator.py) |
| Closure returned from `compile_filter` | same file |
| Named groups, `findall`, walrus in comprehensions | [logic.py](src/log_analyzer/logic.py) |
| Lazy generic generator `filter_lazily[T]` | same file |
| `collections.Counter`, `sorted(key=...)` with tuple keys | `count_by`, `format_report` |
| `argparse` | [main.py](src/log_analyzer/main.py) |

## Patterns

```python
def _scan(source: str) -> Iterator[Token]:     # generator: lazy sequence
    yield Token("paren", "(", pos)

match expr:
    case And(left, right): return evaluate(left, e) and evaluate(right, e)
    case Not(operand): return not evaluate(operand, e)

m = LINE.fullmatch(line); m["level"]           # named groups
(lambda entry: evaluate(tree, entry))          # closure over the parsed tree
dict(re.findall(r"(\w+)=(\S+)", message))      # pairs -> dict
```

## Dialect notes

- Generators and generator expressions are lazy; list comprehensions and `list()` are eager. `itertools` is the standard toolbox for iterator pipelines.
- Python strings are sequences of code points (not UTF-16 units like TypeScript, not bytes like Rust). Slicing and `len` count characters.
- `re.match` anchors at the start only; use `fullmatch` for whole-string checks. `\w` and `\d` are Unicode-aware unless you pass `re.ASCII`.
- `float()` accepts `inf`, `nan` and underscores, so the evaluator validates numbers with its own regex (`to_number`).
- Rust: this is the app where `Box<Expr>` and borrowing slices of the input become the interesting part. Python's garbage collector and reference semantics make it much shorter.
- OS error text differs from Node's (`[Errno 2] No such file...` vs `ENOENT...`).
