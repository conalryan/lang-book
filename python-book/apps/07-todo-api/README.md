# 07 - Todo API (capstone)

A small REST service built with [FastAPI](https://fastapi.tiangolo.com) and [Pydantic](https://docs.pydantic.dev). Focus: project organisation, dependencies, async, visibility and testing. Mirrors the [TypeScript version](../../../typescript-book/apps/07-todo-api/README.md); the routes and JSON shapes are identical.

```sh
uv run todo-api                # http://localhost:3000 (PORT=4000 to change); docs at /docs

curl -XPOST localhost:3000/todos -H 'content-type: application/json' -d '{"title":"learn fastapi"}'
curl 'localhost:3000/todos?done=false'
curl -XPATCH localhost:3000/todos/<id> -H 'content-type: application/json' -d '{"done":true}'
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

Errors are JSON: `{"error": "validation failed", "issues": ["title: ..."]}`. FastAPI's default 422 is mapped to 400 so the contract matches the other ports.

## Architecture

```text
main.py -> io.py (uvicorn)
        -> app.py    HTTP routes: validate input, map errors to status codes
        -> logic.py  TodoService: business rules; InMemoryTodoStore: persistence
           models.py Pydantic models, TodoStore protocol, HttpError
           __init__.py  the package's public API (__all__)
```

Dependencies point inward: `app` knows `logic`, `logic` knows `models`, nothing knows `app`.

## Where to look

| Topic | File |
|-------|------|
| Pydantic models: validation, `Field`, strict types, aliases, `model_validator` | [models.py](src/todo_api/models.py) |
| `async def` and `await`; `Protocol` for the store | `TodoStore`, [logic.py](src/todo_api/logic.py) |
| Constructor injection of clock and id factory | `TodoService` |
| Decorator-based routing, handlers closing over `service` | [app.py](src/todo_api/app.py) |
| Exception handlers (domain, validation, routing errors) | same file |
| Public vs internal names | [\_\_init\_\_.py](src/todo_api/__init__.py) |
| In-process HTTP tests with `TestClient` and a pytest fixture | [test_api.py](tests/test_api.py) |
| Dependencies and console script | [pyproject.toml](pyproject.toml) |

## Dialect notes

- Dependencies live in `pyproject.toml`; `uv add <pkg>` edits it and `uv.lock`. Dev-only tools (pytest, `httpx2`) are in the root `dev` group.
- Visibility is a convention: `_private` names and `__all__` tell humans and linters what is public. There is no `pub`/`export` enforcement.
- `async def` returns a coroutine; the event loop (uvicorn) runs it. Mixing blocking calls into `async` handlers stalls the whole server. FastAPI runs plain `def` handlers in a thread pool, which is why async is optional.
- Pydantic validates at the boundary and gives you typed objects; this is the Zod role in TypeScript and `serde` in Rust. Annotations drive FastAPI too: `body: CreateTodo` means "parse and validate the JSON body".
- `{"title": null}` on PATCH is treated as "not provided" because the field type is `str | None`; Zod rejects it.
- pytest fixtures replace `beforeEach`: a fixture returns a fresh app per test, injected by parameter name.
