from datetime import UTC, datetime, timedelta
from itertools import count

import pytest
from fastapi.testclient import TestClient
from todo_api import InMemoryTodoStore, TodoService, create_app


@pytest.fixture
def client() -> TestClient:
    ids = count(1)
    seconds = count()
    service = TodoService(
        InMemoryTodoStore(),
        new_id=lambda: f"id-{next(ids)}",
        now=lambda: datetime(2024, 1, 1, tzinfo=UTC) + timedelta(seconds=next(seconds)),
    )
    return TestClient(create_app(service))


def titles(response_json: list[dict[str, object]]) -> list[object]:
    return [todo["title"] for todo in response_json]


def test_health(client: TestClient) -> None:
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_unknown_routes_and_methods_are_json_404s(client: TestClient) -> None:
    for response in (client.get("/nope"), client.put("/todos")):
        assert response.status_code == 404
        assert response.json() == {"error": "route not found"}


def test_create_trims_the_title(client: TestClient) -> None:
    response = client.post("/todos", json={"title": "  write docs  "})
    assert response.status_code == 201
    assert response.json() == {
        "id": "id-1",
        "title": "write docs",
        "done": False,
        "createdAt": "2024-01-01T00:00:00.000Z",
    }


@pytest.mark.parametrize(
    ("body", "prefix"),
    [
        ({}, "title: "),
        ({"title": ""}, "title: "),
        ({"title": "   "}, "title: "),
        ({"title": "x" * 201}, "title: "),
        ({"title": 5}, "title: "),
        ({"title": "ok", "extra": 1}, "extra: "),
        ([], "(body): "),
    ],
)
def test_create_rejects_invalid_bodies(client: TestClient, body: object, prefix: str) -> None:
    response = client.post("/todos", json=body)
    assert response.status_code == 400
    payload = response.json()
    assert payload["error"] == "validation failed"
    assert payload["issues"][0].startswith(prefix)


def test_create_rejects_malformed_json(client: TestClient) -> None:
    response = client.post(
        "/todos", content="{not json", headers={"content-type": "application/json"}
    )
    assert response.status_code == 400
    assert response.json() == {"error": "invalid JSON body"}


def test_list_in_creation_order_and_filter_by_done(client: TestClient) -> None:
    client.post("/todos", json={"title": "a"})
    client.post("/todos", json={"title": "b"})
    client.patch("/todos/id-1", json={"done": True})

    assert titles(client.get("/todos").json()) == ["a", "b"]
    assert titles(client.get("/todos?done=true").json()) == ["a"]
    assert titles(client.get("/todos?done=false").json()) == ["b"]


def test_list_rejects_a_bad_filter(client: TestClient) -> None:
    assert client.get("/todos?done=maybe").status_code == 400


def test_get_one_or_404(client: TestClient) -> None:
    client.post("/todos", json={"title": "a"})
    assert client.get("/todos/id-1").status_code == 200
    missing = client.get("/todos/zzz")
    assert missing.status_code == 404
    assert missing.json() == {"error": "todo 'zzz' not found"}


def test_patch_changes_only_provided_fields(client: TestClient) -> None:
    client.post("/todos", json={"title": "a"})
    first = client.patch("/todos/id-1", json={"done": True}).json()
    assert (first["title"], first["done"]) == ("a", True)
    second = client.patch("/todos/id-1", json={"title": "b"}).json()
    assert (second["title"], second["done"]) == ("b", True)


def test_patch_rejects_empty_bodies_and_unknown_todos(client: TestClient) -> None:
    client.post("/todos", json={"title": "a"})
    assert client.patch("/todos/id-1", json={}).status_code == 400
    assert client.patch("/todos/id-1", json={"done": "yes"}).status_code == 400
    assert client.patch("/todos/zzz", json={"done": True}).status_code == 404


def test_delete_once(client: TestClient) -> None:
    client.post("/todos", json={"title": "a"})
    first = client.delete("/todos/id-1")
    assert first.status_code == 204
    assert first.text == ""
    assert client.delete("/todos/id-1").status_code == 404
