# 03 - Contacts

In-memory contact book driven by a tiny command language on stdin. Focus: collections, functions and tuples. State is passed in and returned, never mutated. Mirrors the [TypeScript version](../../../typescript-book/apps/03-contacts/README.md).

```sh
printf 'add Alice 555-1\nadd Adam 555-2\nlist\ngroups\nfind al\ncount\nquit\n' | uv run contacts
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
| Frozen dataclass, `Mapping`, tuple type alias | [models.py](src/contacts/models.py) |
| `dict` copying with `{**book, key: value}` and comprehensions | [logic.py](src/contacts/logic.py) `add_contact`, `remove_contact` |
| Tuple return, starred unpacking | `split_command`, `partition` |
| Default parameter (`limit=None`) | `find_contacts` |
| `sorted(key=...)`, generator expressions | `find_contacts`, `group_by_initial` |
| `dict.setdefault` | `group_by_initial` |
| PEP 695 generic function `partition[T]` | `partition` |
| `match` on a tuple with list patterns | `execute` |

## Patterns

```python
command, *args = line.split() or [""]       # unpack with a starred rest
updated = {**book, key: contact}            # copy + add
{k: v for k, v in book.items() if k != key} # dict comprehension
sorted(items, key=lambda c: c.name.casefold())
match command, args:
    case "add", [name, phone]: ...          # match a tuple of (str, 2-item list)
def find(book, text, limit: int | None = None) -> list[Contact]: ...
```

## Dialect notes

- `list` is the growable array, `tuple` is immutable and fixed, `dict` preserves insertion order, `set` is unordered. `Mapping`/`Sequence` are the read-only abstract types; annotate parameters with them.
- Never use a mutable default argument (`def f(x=[])`): it is created once and shared. Use `None` and create inside, as `run_session` does.
- `sorted()` returns a new list and takes a `key` function; `list.sort()` mutates in place and returns `None`.
- `casefold()` is the correct caseless key; `lower()` is fine for ASCII.
- TypeScript's `Map` is the closest analogue to `dict`; Rust's `HashMap` does not preserve order and moves values in.
