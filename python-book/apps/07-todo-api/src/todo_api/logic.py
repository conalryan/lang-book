"""Business rules and the in-memory store. Free of HTTP concerns apart from `HttpError`."""

from collections.abc import Callable
from datetime import UTC, datetime
from uuid import uuid4

from todo_api.models import CreateTodo, HttpError, Todo, TodoStore, UpdateTodo


class InMemoryTodoStore:
    """Dict-backed store. Matches `TodoStore` structurally, without inheriting from it."""

    def __init__(self) -> None:
        self._todos: dict[str, Todo] = {}

    async def list(self) -> list[Todo]:
        """Every todo."""
        return list(self._todos.values())

    async def get(self, id: str) -> Todo | None:
        """One todo by id."""
        return self._todos.get(id)

    async def save(self, todo: Todo) -> None:
        """Insert or replace."""
        self._todos[todo.id] = todo

    async def delete(self, id: str) -> bool:
        """Delete; True if something was removed."""
        return self._todos.pop(id, None) is not None


class TodoService:
    """Collaborators are injected so tests can fix ids and time."""

    def __init__(
        self,
        store: TodoStore,
        new_id: Callable[[], str] = lambda: str(uuid4()),
        now: Callable[[], datetime] = lambda: datetime.now(UTC),
    ) -> None:
        self._store = store
        self._new_id = new_id
        self._now = now

    async def list(self, done: bool | None = None) -> list[Todo]:
        """Todos, oldest first; optionally only those with the given `done` state."""
        todos = await self._store.list()
        return sorted(
            (t for t in todos if done is None or t.done == done),
            key=lambda t: (t.created_at, t.id),
        )

    async def get(self, id: str) -> Todo:
        """One todo or a 404 `HttpError`."""
        todo = await self._store.get(id)
        if todo is None:
            raise HttpError(404, f"todo '{id}' not found")
        return todo

    async def create(self, data: CreateTodo) -> Todo:
        """Create an open todo."""
        created_at = self._now().astimezone(UTC).isoformat(timespec="milliseconds")
        todo = Todo(
            id=self._new_id(),
            title=data.title,
            done=False,
            created_at=created_at.replace("+00:00", "Z"),
        )
        await self._store.save(todo)
        return todo

    async def update(self, id: str, patch: UpdateTodo) -> Todo:
        """Apply the provided fields only."""
        current = await self.get(id)
        updated = current.model_copy(
            update={
                "title": current.title if patch.title is None else patch.title,
                "done": current.done if patch.done is None else patch.done,
            }
        )
        await self._store.save(updated)
        return updated

    async def delete(self, id: str) -> None:
        """Delete or raise a 404 `HttpError`."""
        if not await self._store.delete(id):
            raise HttpError(404, f"todo '{id}' not found")
