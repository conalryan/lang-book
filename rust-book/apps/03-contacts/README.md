# 03 - Contacts

In-memory contact book driven by a tiny command language on stdin. Focus: collections, functions, tuples and your first real contact with borrowing. Mirrors the [TypeScript](../../../typescript-book/apps/03-contacts/README.md) and [Python](../../../python-book/apps/03-contacts/README.md) versions.

```sh
printf 'add Alice 555-1\nadd Adam 555-2\nlist\ngroups\nfind al\ncount\nquit\n' | cargo run -q --bin contacts
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
| Struct that owns its `String`s, type alias over `BTreeMap` | [models.rs](src/models.rs) |
| `&mut` parameters and the `Entry` API | [logic.rs](src/logic.rs) `add_contact` |
| Returned references and an explicit lifetime `'a` | `find_contacts` |
| Tuple return and destructuring | `split_command`, `partition` |
| `Option<usize>` instead of a sentinel default | `find_contacts(limit)` |
| Generic function with closure parameter | `partition<T>` |
| `match` on `(&str, &[&str])` with slice patterns | `execute` |
| Taking ownership in and returning it back | `run_session` |

## Patterns

```rust
let (command, args) = split_command(line);          // (String, Vec<&str>)
match (command.as_str(), args.as_slice()) {
    ("add", [name, phone]) => { ... }
    ("remove", [name, ..]) => { ... }
    ("remove", []) => usage,
    _ => unknown,
}

pub fn find_contacts<'a>(book: &'a ContactBook, text: &str, limit: Option<usize>) -> Vec<&'a Contact>
//                                  ^ the result borrows from `book`, not from `text`

let (evens, odds) = partition([1, 2, 3, 4, 5], |n| n % 2 == 0);
```

## Dialect notes

- This is the app where ownership shows up. `add_contact(&mut book, ...)` borrows the book exclusively; while that borrow lives, nothing else can read it. TypeScript and Python copy the collection instead, because aliasing makes in-place edits risky.
- `String` owns text; `&str` borrows it. `split_command` returns `&str` slices of `line` without copying, so its result cannot outlive `line` (the compiler checks).
- `BTreeMap` is ordered by key, so "list sorted by name" needs no sort call. `HashMap` is unordered.
- Vec indexing panics when out of range; `.get(i)` returns `Option`. The slice patterns above avoid indexing entirely.
- Rust has no default or keyword arguments; `Option` parameters and builder types fill that gap.
