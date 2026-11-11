# 05 - Library

Checkout and return system. Focus: multiple interacting types, interfaces, inheritance vs composition, generics, dependency injection.

```sh
pnpm start               # scripted scenario with a fake clock
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
| Abstract class, `extends`, `override`, `implements` | [src/models.ts](src/models.ts) `LibraryItem`, `Book`, `Dvd` |
| Generic interface with a bound | `Repository<T extends Entity>` |
| Generic class | [src/logic.ts](src/logic.ts) `InMemoryRepository<T>` |
| Interface as a dependency (structural typing) | `Clock`, `ManualClock` |
| Composition + constructor injection | `LibraryService` |
| Polymorphism | `item.loanDays` in `checkOut` |
| Interface extension | `Loan extends Entity` |

## Patterns

```ts
interface Repository<T extends Entity> { get(id: string): T | undefined; ... }
class InMemoryRepository<T extends Entity> implements Repository<T> { ... }

abstract class LibraryItem { abstract readonly loanDays: number; }
class Book extends LibraryItem { override readonly loanDays = 14; }
```

## Inheritance or composition?

`Book`/`Dvd` use inheritance because they genuinely share identity and vary one property. `LibraryService` uses composition because it needs several independent collaborators, which is easy to swap in tests (`ManualClock`, in-memory repos).

## Dialect notes

- Interfaces are structural: any object with the right shape satisfies one. `implements` only asserts that at compile time. Rust traits are explicit; Python `Protocol` is the closest equivalent.
- `private` is compile-time only; `#field` is real runtime privacy.
- Generics are erased at runtime, so you cannot do `new T()` or `T.name`. Rust monomorphises generics.
- Single inheritance only; mix behaviours through interfaces or composition.
