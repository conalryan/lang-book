# 07 - Todo API (capstone)

A small REST service built with [Hono](https://hono.dev), validated with [Zod](https://zod.dev). Focus: project organisation, dependencies, async, visibility, testing.

```sh
pnpm start                     # http://localhost:3000 (PORT=4000 to change)
pnpm dev                       # restart on file changes

curl -XPOST localhost:3000/todos -H 'content-type: application/json' -d '{"title":"learn hono"}'
curl localhost:3000/todos?done=false
curl -XPATCH localhost:3000/todos/<id> -H 'content-type: application/json' -d '{"done":true}'
curl -XDELETE localhost:3000/todos/<id>
```

| Method | Path | Result |
|--------|------|--------|
| GET | `/health` | `200 {"status":"ok"}` |
| GET | `/todos[?done=true\|false]` | `200` list, oldest first |
| POST | `/todos` `{title}` | `201` todo |
| GET | `/todos/:id` | `200` todo or `404` |
| PATCH | `/todos/:id` `{title?, done?}` | `200` todo, `400`, `404` |
| DELETE | `/todos/:id` | `204` or `404` |

Errors are JSON: `{"error": "validation failed", "issues": ["title: ..."]}`.

## Architecture

```text
main.ts -> io.ts (listen, signals)
        -> app.ts   HTTP routes: parse + validate input, map errors to status codes
        -> logic.ts TodoService: business rules; InMemoryTodoStore: persistence
           models.ts Zod schemas, derived types, TodoStore interface, HttpError
           index.ts  the package's public API
```

Dependencies point inward: `app` knows `logic`, `logic` knows `models`, nothing knows `app`.

## Where to look

| Topic | File |
|-------|------|
| Runtime schema -> static type with `z.infer` | [src/models.ts](src/models.ts) |
| Async interfaces and `Promise<T>` | `TodoStore`, `InMemoryTodoStore` |
| Constructor injection of clock and id generator | [src/logic.ts](src/logic.ts) |
| Routing, middleware-style error handling | [src/app.ts](src/app.ts) |
| Public vs internal modules (`export`, barrel, `package.json` `exports`) | [src/index.ts](src/index.ts) |
| Graceful shutdown | [src/io.ts](src/io.ts) |
| In-process HTTP tests with `app.request` (no sockets) | [test/api.test.ts](test/api.test.ts) |
| Declaration output for consumers | `tsconfig.build.json` (`declaration: true`) |

## Dialect notes

- Dependencies are declared as `catalog:` and versions live in `pnpm-workspace.yaml`. `dependencies` vs `devDependencies` matter for published packages, less for apps.
- Visibility is per module: only `export`ed names are visible, and `package.json` `exports` blocks deep imports. There is no `pub(crate)`; use the barrel file.
- `async` functions always return a `Promise`, and thrown errors become rejections. Unhandled rejections crash modern Node.
- Rust equivalent: axum + serde + `tower` tests; Python: FastAPI + pydantic + `TestClient`.
- Doc comments are TSDoc (`/** ... */`); types in comments are not needed.
