# 05 - Library

Checkout and return system. Focus: multiple interacting types, traits, generics, trait objects and composition. Mirrors the [TypeScript](../../../typescript-book/apps/05-library/README.md) and [Python](../../../python-book/apps/05-library/README.md) versions.

```sh
cargo run -q --bin library-demo      # scripted scenario with a fake clock
```

```text
Checked out 'Dune' to Ada, due 2024-01-15
Checked out 'Alien' to Ada, due 2024-01-08
error: Ada has reached the limit of 2 loans
error: 'Dune' is already checked out
Returned 'Alien' on time
Returned 'Dune' 3 days late, fee $0.75
error: 'Dune' is not checked out
```

Rules: books are lent for 14 days, DVDs for 7, members may hold 2 loans, the late fee is $0.25 per day.

## Where to look

| Topic | File |
|-------|------|
| Traits with default-free methods, supertrait (`LibraryItem: Entity`) | [models.rs](src/models.rs) |
| `impl Trait for Struct` (no inheritance) | `Book`, `Dvd` |
| Generic trait with a bound: `Repository<T: Entity>` | `Repository` |
| Blanket impl so `Box<dyn LibraryItem>` is an `Entity` | `impl<T: Entity + ?Sized> Entity for Box<T>` |
| Generic struct implementing the trait | [logic.rs](src/logic.rs) `InMemoryRepository<T>` |
| Composition, generics with `where` bounds, `&'a dyn Clock` | `LibraryService` |
| Polymorphism through a trait object | `item.loan_days()` in `check_out` |
| Interior mutability with `Cell` | `ManualClock` |
| A small hand-written date type | [date.rs](src/date.rs) |

## Patterns

```rust
pub trait Repository<T: Entity> { fn get(&self, id: &str) -> Option<&T>; ... }
impl<T: Entity> Repository<T> for InMemoryRepository<T> { ... }

pub trait LibraryItem: Entity { fn loan_days(&self) -> u32; }   // supertrait
let items: Vec<Box<dyn LibraryItem>> = vec![Box::new(Book::new(..)), Box::new(Dvd::new(..))];

pub struct LibraryService<'a, I, M, L> { items: I, clock: &'a dyn Clock, .. }
impl<'a, I, M, L> LibraryService<'a, I, M, L> where I: Repository<Box<dyn LibraryItem>>, .. { }
```

## Static or dynamic dispatch?

The repositories are generic parameters (`I`, `M`, `L`): the compiler generates specialised code, no runtime lookup. The items are `Box<dyn LibraryItem>` because one collection must hold *different* concrete types, which requires a vtable. The clock is `&dyn Clock` to avoid a fourth type parameter. This is the trade-off TypeScript and Python never ask you to make.

## Dialect notes

- No inheritance: `Book` and `Dvd` share behaviour through a trait, not a base class. Fields are not inherited, so each struct declares its own `id` and `title`.
- Traits are implemented explicitly (`impl Trait for Type`); Python `Protocol` and TypeScript `interface` are structural.
- The service stores `&'a dyn Clock`: the `'a` lifetime says "the clock must outlive the service". The demo still advances time via `Cell`, since `&self` methods cannot otherwise mutate.
- `Repository::add` takes the item by value: it *moves* into the repository and the caller can no longer use it.
- `impl Trait` in argument position (`find(&self, predicate: impl Fn(&T) -> bool)`) is generic sugar; it also makes the trait not usable as `dyn`.
- The standard library has no date type. `chrono` or `time` are the usual crates (app 7 uses `chrono`).
