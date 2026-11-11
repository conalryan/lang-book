"""Capstone: a small REST API with FastAPI.

Public API: `create_app`, `TodoService`, `InMemoryTodoStore` and the model types.
Anything not listed in `__all__` is internal, even though Python cannot enforce that.
"""

from todo_api.app import create_app
from todo_api.logic import InMemoryTodoStore, TodoService
from todo_api.models import CreateTodo, Todo, TodoStore, UpdateTodo

__all__ = [
    "CreateTodo",
    "InMemoryTodoStore",
    "Todo",
    "TodoService",
    "TodoStore",
    "UpdateTodo",
    "create_app",
]
