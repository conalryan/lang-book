//! HTTP layer: handlers translate HTTP to service calls and errors to status codes.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use serde_json::{Value, json};

use crate::logic::TodoService;
use crate::models::{ApiError, CreateTodo, Todo, TodoStore, UpdateTodo};

type Shared<S> = State<Arc<TodoService<S>>>;

fn read_json(body: &Bytes) -> Result<Value, ApiError> {
    serde_json::from_slice(body)
        .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, "invalid JSON body"))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn list_todos<S: TodoStore + 'static>(
    State(service): Shared<S>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Todo>>, ApiError> {
    let done = match params.get("done").map(String::as_str) {
        None => None,
        Some("true") => Some(true),
        Some("false") => Some(false),
        Some(_) => {
            return Err(ApiError::validation(vec![
                "done: expected 'true' or 'false'".to_string(),
            ]));
        }
    };
    Ok(Json(service.list(done).await))
}

async fn create_todo<S: TodoStore + 'static>(
    State(service): Shared<S>,
    body: Bytes,
) -> Result<(StatusCode, Json<Todo>), ApiError> {
    let input = CreateTodo::from_json(&read_json(&body)?).map_err(ApiError::validation)?;
    Ok((StatusCode::CREATED, Json(service.create(input).await)))
}

async fn get_todo<S: TodoStore + 'static>(
    State(service): Shared<S>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, ApiError> {
    Ok(Json(service.get(&id).await?))
}

async fn update_todo<S: TodoStore + 'static>(
    State(service): Shared<S>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<Todo>, ApiError> {
    let patch = UpdateTodo::from_json(&read_json(&body)?).map_err(ApiError::validation)?;
    Ok(Json(service.update(&id, patch).await?))
}

async fn delete_todo<S: TodoStore + 'static>(
    State(service): Shared<S>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn route_not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "route not found")
}

/// Build the router around a service. The `Arc` lets every request share it cheaply.
pub fn create_app<S: TodoStore + 'static>(service: TodoService<S>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", get(list_todos::<S>).post(create_todo::<S>))
        .route(
            "/todos/{id}",
            get(get_todo::<S>)
                .patch(update_todo::<S>)
                .delete(delete_todo::<S>),
        )
        .fallback(route_not_found)
        // An unknown method is also "no such route" here.
        .method_not_allowed_fallback(route_not_found)
        .with_state(Arc::new(service))
}
