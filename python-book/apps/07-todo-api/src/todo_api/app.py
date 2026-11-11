"""HTTP layer: routes translate HTTP to service calls and errors to status codes."""

from typing import Literal

from fastapi import FastAPI, Request, Response
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHTTPException

from todo_api.logic import TodoService
from todo_api.models import CreateTodo, HttpError, Todo, UpdateTodo


def create_app(service: TodoService) -> FastAPI:
    """Build the app around a service. Handlers close over `service`."""
    app = FastAPI(title="Todo API")

    @app.get("/health")
    async def health() -> dict[str, str]:
        return {"status": "ok"}

    @app.get("/todos")
    async def list_todos(done: Literal["true", "false"] | None = None) -> list[Todo]:
        return await service.list(None if done is None else done == "true")

    @app.post("/todos", status_code=201)
    async def create_todo(body: CreateTodo) -> Todo:
        return await service.create(body)

    @app.get("/todos/{todo_id}")
    async def get_todo(todo_id: str) -> Todo:
        return await service.get(todo_id)

    @app.patch("/todos/{todo_id}")
    async def update_todo(todo_id: str, body: UpdateTodo) -> Todo:
        return await service.update(todo_id, body)

    @app.delete("/todos/{todo_id}", status_code=204)
    async def delete_todo(todo_id: str) -> Response:
        await service.delete(todo_id)
        return Response(status_code=204)

    @app.exception_handler(HttpError)
    async def on_http_error(_: Request, error: HttpError) -> JSONResponse:
        body: dict[str, object] = {"error": error.message}
        if error.issues:
            body["issues"] = error.issues
        return JSONResponse(body, status_code=error.status)

    @app.exception_handler(RequestValidationError)
    async def on_validation_error(_: Request, error: RequestValidationError) -> JSONResponse:
        problems = error.errors()
        if any(problem["type"] == "json_invalid" for problem in problems):
            return JSONResponse({"error": "invalid JSON body"}, status_code=400)
        issues = [
            f"{'.'.join(map(str, problem['loc'][1:])) or '(body)'}: {problem['msg']}"
            for problem in problems
        ]
        return JSONResponse({"error": "validation failed", "issues": issues}, status_code=400)

    @app.exception_handler(StarletteHTTPException)
    async def on_not_found(_: Request, error: StarletteHTTPException) -> JSONResponse:
        if error.status_code in (404, 405):  # an unknown method is also "no such route" here
            return JSONResponse({"error": "route not found"}, status_code=404)
        return JSONResponse({"error": str(error.detail)}, status_code=error.status_code)

    return app
