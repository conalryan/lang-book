"""Schemas, derived types and the persistence interface."""

from typing import Protocol, Self

from pydantic import BaseModel, ConfigDict, Field, StrictBool, StrictStr, model_validator
from pydantic.alias_generators import to_camel


class Todo(BaseModel):
    """A stored todo. Serialises with camelCase keys (`createdAt`) to match the other ports."""

    model_config = ConfigDict(frozen=True, alias_generator=to_camel, populate_by_name=True)

    id: str
    title: str
    done: bool
    created_at: str


class CreateTodo(BaseModel):
    """Request body for POST /todos. Pydantic models validate at runtime *and* type the code."""

    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    title: StrictStr = Field(min_length=1, max_length=200)


class UpdateTodo(BaseModel):
    """Request body for PATCH /todos/{id}. At least one field is required."""

    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    title: StrictStr | None = Field(default=None, min_length=1, max_length=200)
    done: StrictBool | None = None

    @model_validator(mode="after")
    def _require_a_field(self) -> Self:
        if self.title is None and self.done is None:
            raise ValueError("provide at least one of: title, done")
        return self


class TodoStore(Protocol):
    """Persistence boundary. Async so a database-backed store can replace the in-memory one."""

    async def list(self) -> list[Todo]:
        """Every todo."""
        ...

    async def get(self, id: str) -> Todo | None:
        """One todo by id."""
        ...

    async def save(self, todo: Todo) -> None:
        """Insert or replace."""
        ...

    async def delete(self, id: str) -> bool:
        """Delete; True if something was removed."""
        ...


class HttpError(Exception):
    """An error that knows which HTTP status it maps to."""

    def __init__(self, status: int, message: str, issues: list[str] | None = None) -> None:
        super().__init__(message)
        self.status = status
        self.message = message
        self.issues = issues or []
