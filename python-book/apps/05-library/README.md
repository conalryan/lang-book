# 05 - Library

Checkout and return system. Focus: multiple interacting types, protocols, inheritance vs composition, generics and dependency injection. Mirrors the [TypeScript version](../../../typescript-book/apps/05-library/README.md).

```sh
uv run library-demo               # scripted scenario with a fake clock
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
| Abstract base class, `@abstractmethod`, abstract `@property`, `super().__init__` | [models.py](src/library/models.py) `LibraryItem`, `Book`, `Dvd` |
| `Protocol` (structural interface) | `Entity`, `Clock` |
| Generic protocol with a bound: `Repository[T: Entity]` | `Repository` |
| Generic class implementing it without inheriting | [logic.py](src/library/logic.py) `InMemoryRepository[T]` |
| Composition + constructor injection | `LibraryService` |
| Polymorphism | `item.loan_days` in `check_out` |
| Class vs plain-data types | `Member`, `Loan` (frozen dataclasses) vs `Book` (class) |

## Patterns

```python
class Repository[T: Entity](Protocol):
    def get(self, id: str) -> T | None: ...

class InMemoryRepository[T: Entity]:       # no base class: matching the shape is enough
    ...

class LibraryItem(ABC):
    @property
    @abstractmethod
    def loan_days(self) -> int: ...

items = InMemoryRepository[LibraryItem]()  # parametrise at construction
```

## Inheritance or composition?

`Book`/`Dvd` use inheritance because they genuinely share identity and vary one property. `LibraryService` uses composition: it needs several independent collaborators, which tests replace (`ManualClock`, in-memory repositories).

## Dialect notes

- `Protocol` is structural (like TypeScript interfaces); `ABC` is nominal and enforced at instantiation (`LibraryItem("x", "y")` raises `TypeError`). Rust traits are explicit; Python gives you both styles.
- Python supports multiple inheritance (MRO). Prefer composition and Protocols; mixins are the legitimate use.
- Generics are erased at runtime, like TypeScript. `InMemoryRepository[LibraryItem]()` is only a hint for the type checker.
- `id` shadows a builtin as a parameter; harmless here, but a reason ruff's `A` rules exist.
- `datetime` arithmetic gives `timedelta`; dividing two `timedelta`s gives a float, which is how the late days are computed.
- Dunder methods (`__init__`, `__eq__`) are the "operator overloading" hooks; dataclasses generate them.
