# 03 - Contacts

In-memory contact book driven by a tiny command language on stdin. Focus: collections, functions and tuples. State is passed in and returned, never mutated.

```sh
printf 'add Alice 555-1\nadd Adam 555-2\nlist\ngroups\nfind al\ncount\nquit\n' | pnpm start
```

| Command | Output |
|---------|--------|
| `add <name> <phone>` | `Added <name>` or `error: <name> already exists` |
| `remove <name>` | `Removed <name>` or `error: <name> not found` |
| `find <text>` | matching `<name> <phone>` lines, or `(no matches)` |
| `list` | all contacts sorted by name, or `(no contacts)` |
| `groups` | `A: Adam, Alice` per initial |
| `count` | `N contact(s)` |
| `help`, `quit` | usage / `Bye` |

## Where to look

| Topic | File |
|-------|------|
| `Map`, `ReadonlyMap`, immutability by copying | [src/logic.ts](src/logic.ts) `addContact`, `removeContact` |
| Tuples with labels, `readonly` tuples | [src/models.ts](src/models.ts) `CommandResult` |
| Tuple return and destructuring | `splitCommand`, `partition` |
| Default and rest parameters | `findContacts`, `splitCommand` |
| Higher-order array methods | `findContacts`, `groupByInitial` |
| Generic function | `partition<T>` |

## Patterns

```ts
const [command = "", ...args] = line.split(/\s+/);   // destructure with default + rest
const next = new Map([...book, [key, contact]]);     // spread to copy
const names = groups.get(k) ?? [];                   // Map.get returns T | undefined
function f(limit = Infinity) {}                      // default parameter
```

## Dialect notes

- Arrays are `T[]` (growable). Tuples are a type-level view of an array: `[string, number]`.
- `Map` preserves insertion order and uses `===`/SameValueZero for keys; plain objects stringify keys.
- `sort()` mutates and sorts as strings by default; always pass a comparator.
- Python: `dict`/`list`/`tuple` are runtime types; Rust: `HashMap` is unordered and ownership moves values in.
