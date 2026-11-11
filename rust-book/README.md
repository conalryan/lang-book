# Rust Book

Seven small, runnable apps that walk through Rust one concept cluster at a time. Each app mirrors the [TypeScript reference](../typescript-book/README.md) and the [Python port](../python-book/README.md): same layout, same commands, same messages, so the code can be diffed across languages.

## Apps

| # | App | Command | New concepts |
|---|-----|---------|--------------|
| 1 | [calculator](./apps/01-calculator) | `calc` | `main`, primitives, shadowing, `as` casts, `enum` + `impl`, slice patterns |
| 2 | [guessing-game](./apps/02-guessing-game) | `guess` | `if`/`match`/`while`/`for`, `Ordering`, closures (`FnMut`, `move`), `impl Trait` |
| 3 | [contacts](./apps/03-contacts) | `contacts` | `Vec`, `BTreeMap`, `&mut`, borrowing, lifetimes, tuples, slices |
| 4 | [temperature-logger](./apps/04-temperature-logger) | `templog` | structs, tagged enums, `Option`, `Result` and `?`, `let ... else` |
| 5 | [library](./apps/05-library) | `library-demo` | traits, supertraits, `dyn`, generics with bounds, `Cell`, composition |
| 6 | [log-analyzer](./apps/06-log-analyzer) | `log-analyzer` | `Iterator`, `Box<Expr>`, recursive descent, pattern matching, closures |
| 7 | [todo-api](./apps/07-todo-api) | `todo-api` | crates, `async`/tokio, axum, serde, visibility, integration tests |

## Toolchain

| Concern | Tool |
|---------|------|
| Language version | Rust 1.99.1, pinned in `rust-toolchain.toml` (rustup installs it on first use) |
| Package manager + workspace | Cargo workspace (`apps/*`), shared versions in `[workspace.dependencies]`, one `Cargo.lock` |
| Build caching | Cargo's incremental compilation and the shared `target/` directory |
| Tests | [cargo-nextest](https://nexte.st) (parallel, per-test processes) plus `cargo test --doc` |
| Coverage | [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) (LLVM source-based) |
| Format | rustfmt (`rustfmt.toml`) |
| Lint + typecheck | Clippy with `pedantic`, configured once in `[workspace.lints]`, warnings denied in CI |
| Task runner | `make` (thin wrapper over cargo) |

## Getting started

```sh
brew install rustup cargo-nextest cargo-llvm-cov
export PATH="$(brew --prefix rustup)/bin:$PATH"   # exposes cargo and rustc; the pinned toolchain installs on first use

make check             # rustfmt --check, clippy -D warnings, nextest, doctests
make coverage          # LLVM coverage report
make fmt

cargo run -p calculator --bin calc -- 3 + 4
cargo nextest run -p contacts
```

## Layout

```text
apps/NN-name/
  src/
    lib.rs           declares the modules; everything testable lives in the library
    models.rs        types: structs, enums, traits (the data)
    logic.rs         pure functions and services (the behaviour)
    logic/tests.rs   unit tests, compiled only under `cfg(test)`, can see private items
    io.rs            the only module that touches stdin/stdout/files/network
    main.rs          the binary: wiring only
  tests/             integration tests that run the real binary or use only the public API
  Cargo.toml         package, [[bin]] name (the CLI command), `[lints] workspace = true`
```

`main.rs` and `io.rs` are excluded from coverage because they are thin wiring. Apps 5 to 7 add modules where a concept deserves its own file (`date.rs`, `demo.rs`, `lexer.rs`, `parser.rs`, `evaluator.rs`, `app.rs`).

## Cross-language parity

Commands, messages and exit codes match the TypeScript and Python apps. 55 CLI scenarios (including error paths and seeded games) produce byte-identical output in all three languages. Deliberate differences:

- `calc tour` is Rust-specific, because its job is to show language quirks.
- OS error text differs (a missing file in app 6).
- Validation issue text in app 7 differs after the `field: ` prefix.
- App 7 binds to `127.0.0.1` rather than all interfaces.

## Rust dialect notes

- **Ownership decides the API shape.** Functions that only read take `&T`, functions that edit take `&mut T`, and functions that consume take `T`. Apps 3 and 5 show where TypeScript and Python copy or alias instead.
- **No exceptions, no null.** Failure is `Result<T, E>`, absence is `Option<T>`, and `?` propagates. `panic!`/`unwrap` are for bugs and tests.
- **Enums carry data.** Tagged enums replace class hierarchies, discriminated unions and `Result` classes; `match` must be exhaustive.
- **Traits are explicit.** There is no structural typing: a type implements a trait with an `impl` block. Generics are monomorphised (static dispatch), `dyn Trait` gives dynamic dispatch.
- **No inheritance.** Share behaviour with traits (including default methods and supertraits) and composition.
- **Strings are two types**: `String` (owned) and `&str` (borrowed view). `len()` counts bytes; use `chars()` for characters.
- **Closures are three traits** (`Fn`, `FnMut`, `FnOnce`) depending on how they use captured values.
- **Privacy is module-level and default-private.** `pub`, `pub(crate)` and `pub use` re-exports define a crate's API, and the compiler enforces it.
- **Tests live next to the code.** Unit tests are a `#[cfg(test)]` module; integration tests in `tests/` see only the public API.
- **Rust has no decorators**; attributes (`#[derive(...)]`, `#[tokio::main]`) and procedural macros fill that role.
