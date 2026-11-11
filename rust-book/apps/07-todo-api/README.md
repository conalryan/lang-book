# 07 - Todo API (capstone)

A small REST service built with [axum](https://docs.rs/axum) on [tokio](https://tokio.rs), with [serde_json](https://docs.rs/serde_json) for JSON. Focus: crates and dependencies, async, visibility and testing. Mirrors the [TypeScript](../../../typescript-book/apps/07-todo-api/README.md) and [Python](../../../python-book/apps/07-todo-api/README.md) versions; routes and JSON shapes are identical.

```sh
cargo run -q --bin todo-api       # http://127.0.0.1:3000 (PORT=4000 to change)

curl -XPOST localhost:3000/todos -H 'content-type: application/json' -d '{"title":"learn axum"}'
curl 'localhost:3000/todos?done=false'
curl -XPATCH localhost:3000/todos/<id> -d '{"done":true}'
curl -XDELETE localhost:3000/todos/<id>
```

| Method | Path | Result |
|--------|------|--------|
| GET | `/health` | `200 {"status":"ok"}` |
| GET | `/todos[?done=true\|false]` | `200` list, oldest first |
| POST | `/todos` `{title}` | `201` todo |
| GET | `/todos/{id}` | `200` todo or `404` |
| PATCH | `/todos/{id}` `{title?, done?}` | `200` todo, `400`, `404` |
| DELETE | `/todos/{id}` | `204` or `404` |

Errors are JSON: `{"error": "validation failed", "issues": ["title: ..."]}`.

## Architecture

```text
main.rs -> io.rs   bind a port, graceful shutdown
        -> app.rs  routes: parse + validate input, map errors to status codes
           logic.rs  TodoService<S>: business rules; InMemoryTodoStore: persistence
           models.rs types, validation, TodoStore trait, ApiError
           lib.rs    private modules + `pub use`: the crate's public API
```

Dependencies point inward: `app` knows `logic`, `logic` knows `models`, nothing knows `app`.

## Where to look

| Topic | File |
|-------|------|
| `[workspace.dependencies]` and `dep.workspace = true` | [Cargo.toml](Cargo.toml), [../../Cargo.toml](../../Cargo.toml) |
| Private modules + `pub use` re-exports | [lib.rs](src/lib.rs) |
| `#[derive(Serialize)]`, `#[serde(rename_all)]` | [models.rs](src/models.rs) `Todo` |
| Manual validation of `serde_json::Value` | `CreateTodo::from_json`, `UpdateTodo::from_json` |
| `IntoResponse` for an error type, `?` in handlers | `ApiError`, [app.rs](src/app.rs) |
| `async fn` in a trait (`impl Future + Send`) | `TodoStore` |
| `Arc`, async `Mutex`, builder methods | [logic.rs](src/logic.rs) |
| Extractors: `State`, `Path`, `Query`, `Bytes` | [app.rs](src/app.rs) |
| `#[tokio::main]`, signals, `select!` | [main.rs](src/main.rs), [io.rs](src/io.rs) |
| In-process HTTP tests via `tower::ServiceExt::oneshot` | [tests/api.rs](tests/api.rs) |

## Dialect notes

- Dependencies are declared per crate; versions live once in the workspace. `cargo add` edits the manifest and `Cargo.lock` pins the exact tree.
- Visibility is checked by the compiler: `mod app;` is private, so consumers can only use what `lib.rs` re-exports. `pub(crate)` sits between private and `pub`.
- `async fn` returns a future that does nothing until awaited, and needs a runtime (`tokio`). Futures that cross threads must be `Send`, which is why the store trait spells out `+ Send`.
- Shared state is `Arc<TodoService<S>>`: `Arc` for shared ownership across requests, an async `Mutex` for interior mutability. Rust forces you to say how data is shared; Node and Python hide it behind the GIL / event loop.
- The request body is taken as raw `Bytes` and parsed by hand, so malformed JSON and validation errors produce the exact JSON shape of the other ports (axum's `Json` extractor has its own rejections).
- `serde` is Rust's answer to Zod and Pydantic: derive-based, compile-time, but validation rules (length, trimming) are still your code.
- Attributes like `#[derive]`, `#[serde(..)]` and `#[tokio::main]` are Rust's closest relative to decorators; they run at compile time.
- Integration tests in `tests/` can only see the public API, so they test the same surface a consumer would.
