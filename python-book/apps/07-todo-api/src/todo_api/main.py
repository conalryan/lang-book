"""Entry point: wiring only."""

import os

from todo_api.app import create_app
from todo_api.io import start_server
from todo_api.logic import InMemoryTodoStore, TodoService


def main() -> int:
    """Start the server on $PORT (default 3000)."""
    start_server(create_app(TodoService(InMemoryTodoStore())), int(os.environ.get("PORT", "3000")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
