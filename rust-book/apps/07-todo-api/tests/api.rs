//! Black-box tests: only the crate's public API is visible here, like any other consumer.

use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::{Duration, TimeZone, Utc};
use serde_json::{Value, json};
use todo_api::{InMemoryTodoStore, TodoService, create_app};
use tower::ServiceExt;

fn app() -> Router {
    let ids = AtomicUsize::new(0);
    let ticks = AtomicI64::new(0);
    let service = TodoService::new(InMemoryTodoStore::default())
        .with_id_generator(move || format!("id-{}", ids.fetch_add(1, Ordering::SeqCst) + 1))
        .with_clock(move || {
            let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
            start + Duration::seconds(ticks.fetch_add(1, Ordering::SeqCst))
        });
    create_app(service)
}

/// Returns the status and the parsed JSON body (`Null` for an empty body).
async fn send(app: &Router, method: &str, path: &str, body: Option<&str>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn titles(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|t| t["title"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn health() {
    let (status, body) = send(&app(), "GET", "/health", None).await;
    assert_eq!((status, body), (StatusCode::OK, json!({ "status": "ok" })));
}

#[tokio::test]
async fn unknown_routes_and_methods_are_json_404s() {
    let app = app();
    for (method, path) in [("GET", "/nope"), ("PUT", "/todos")] {
        let (status, body) = send(&app, method, path, None).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, json!({ "error": "route not found" }))
        );
    }
}

#[tokio::test]
async fn create_trims_the_title() {
    let (status, body) = send(
        &app(),
        "POST",
        "/todos",
        Some(r#"{"title":"  write docs  "}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body,
        json!({ "id": "id-1", "title": "write docs", "done": false, "createdAt": "2024-01-01T00:00:00.000Z" })
    );
}

#[tokio::test]
async fn create_rejects_invalid_bodies() {
    let app = app();
    let long = format!(r#"{{"title":"{}"}}"#, "x".repeat(201));
    let cases = [
        ("{}", "title: "),
        (r#"{"title":""}"#, "title: "),
        (r#"{"title":"   "}"#, "title: "),
        (long.as_str(), "title: "),
        (r#"{"title":5}"#, "title: "),
        (r#"{"title":"ok","extra":1}"#, "(body): "),
        ("[]", "(body): "),
    ];
    for (input, prefix) in cases {
        let (status, body) = send(&app, "POST", "/todos", Some(input)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{input}");
        assert_eq!(body["error"], "validation failed");
        assert!(
            body["issues"][0].as_str().unwrap().starts_with(prefix),
            "{input}"
        );
    }
}

#[tokio::test]
async fn create_rejects_malformed_json() {
    let app = app();
    for input in ["{not json", ""] {
        let (status, body) = send(&app, "POST", "/todos", Some(input)).await;
        assert_eq!(
            (status, body),
            (
                StatusCode::BAD_REQUEST,
                json!({ "error": "invalid JSON body" })
            )
        );
    }
}

#[tokio::test]
async fn list_in_creation_order_and_filter_by_done() {
    let app = app();
    send(&app, "POST", "/todos", Some(r#"{"title":"a"}"#)).await;
    send(&app, "POST", "/todos", Some(r#"{"title":"b"}"#)).await;
    send(&app, "PATCH", "/todos/id-1", Some(r#"{"done":true}"#)).await;

    assert_eq!(
        titles(&send(&app, "GET", "/todos", None).await.1),
        ["a", "b"]
    );
    assert_eq!(
        titles(&send(&app, "GET", "/todos?done=true", None).await.1),
        ["a"]
    );
    assert_eq!(
        titles(&send(&app, "GET", "/todos?done=false", None).await.1),
        ["b"]
    );
}

#[tokio::test]
async fn list_rejects_a_bad_filter() {
    assert_eq!(
        send(&app(), "GET", "/todos?done=maybe", None).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn get_one_or_404() {
    let app = app();
    send(&app, "POST", "/todos", Some(r#"{"title":"a"}"#)).await;
    assert_eq!(
        send(&app, "GET", "/todos/id-1", None).await.0,
        StatusCode::OK
    );
    let (status, body) = send(&app, "GET", "/todos/zzz", None).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::NOT_FOUND,
            json!({ "error": "todo 'zzz' not found" })
        )
    );
}

#[tokio::test]
async fn patch_changes_only_provided_fields() {
    let app = app();
    send(&app, "POST", "/todos", Some(r#"{"title":"a"}"#)).await;
    let (_, first) = send(&app, "PATCH", "/todos/id-1", Some(r#"{"done":true}"#)).await;
    assert_eq!(
        (&first["title"], &first["done"]),
        (&json!("a"), &json!(true))
    );
    let (_, second) = send(&app, "PATCH", "/todos/id-1", Some(r#"{"title":"b"}"#)).await;
    assert_eq!(
        (&second["title"], &second["done"]),
        (&json!("b"), &json!(true))
    );
}

#[tokio::test]
async fn patch_rejects_empty_bodies_and_unknown_todos() {
    let app = app();
    send(&app, "POST", "/todos", Some(r#"{"title":"a"}"#)).await;
    assert_eq!(
        send(&app, "PATCH", "/todos/id-1", Some("{}")).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(&app, "PATCH", "/todos/id-1", Some(r#"{"done":"yes"}"#))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(&app, "PATCH", "/todos/zzz", Some(r#"{"done":true}"#))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn delete_once() {
    let app = app();
    send(&app, "POST", "/todos", Some(r#"{"title":"a"}"#)).await;
    assert_eq!(
        send(&app, "DELETE", "/todos/id-1", None).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        send(&app, "DELETE", "/todos/id-1", None).await.0,
        StatusCode::NOT_FOUND
    );
}
