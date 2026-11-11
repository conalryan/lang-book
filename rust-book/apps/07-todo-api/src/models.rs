//! Types, request validation and the persistence interface.

use std::fmt;
use std::future::Future;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Value, json};

/// A stored todo. `Serialize` plus `rename_all` produces camelCase JSON (`createdAt`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub created_at: String,
}

/// A validated `POST /todos` body. Fields are public, but the only way to *obtain* one from
/// user input is [`CreateTodo::from_json`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTodo {
    pub title: String,
}

/// A validated `PATCH /todos/{id}` body. At least one field is `Some`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}

fn title_from(value: &Value) -> Result<String, String> {
    let Some(text) = value.as_str() else {
        return Err("title: expected a string".to_string());
    };
    let title = text.trim();
    if title.is_empty() {
        Err("title: must not be empty".to_string())
    } else if title.chars().count() > 200 {
        Err("title: must be at most 200 characters".to_string())
    } else {
        Ok(title.to_string())
    }
}

/// Strict objects: any key outside `allowed` is reported.
fn unknown_keys(body: &serde_json::Map<String, Value>, allowed: &[&str]) -> Vec<String> {
    body.keys()
        .filter(|key| !allowed.contains(&key.as_str()))
        .map(|key| format!("(body): unrecognized key '{key}'"))
        .collect()
}

impl CreateTodo {
    /// Validate a JSON body, returning every problem found as `field: message`.
    pub fn from_json(body: &Value) -> Result<Self, Vec<String>> {
        let Some(object) = body.as_object() else {
            return Err(vec!["(body): expected an object".to_string()]);
        };
        let mut issues = unknown_keys(object, &["title"]);

        let title = match object.get("title").map(title_from) {
            None => {
                issues.push("title: required".to_string());
                None
            }
            Some(Ok(title)) => Some(title),
            Some(Err(issue)) => {
                issues.push(issue);
                None
            }
        };
        match title {
            Some(title) if issues.is_empty() => Ok(Self { title }),
            _ => Err(issues),
        }
    }
}

impl UpdateTodo {
    /// Validate a JSON body, returning every problem found as `field: message`.
    pub fn from_json(body: &Value) -> Result<Self, Vec<String>> {
        let Some(object) = body.as_object() else {
            return Err(vec!["(body): expected an object".to_string()]);
        };
        let mut issues = unknown_keys(object, &["title", "done"]);

        let title = match object.get("title").map(title_from) {
            None => None,
            Some(Ok(title)) => Some(title),
            Some(Err(issue)) => {
                issues.push(issue);
                None
            }
        };
        let done = match object.get("done") {
            None => None,
            Some(Value::Bool(done)) => Some(*done),
            Some(_) => {
                issues.push("done: expected a boolean".to_string());
                None
            }
        };
        if issues.is_empty() && title.is_none() && done.is_none() {
            issues.push("(body): provide at least one of: title, done".to_string());
        }
        if issues.is_empty() {
            Ok(Self { title, done })
        } else {
            Err(issues)
        }
    }
}

/// Persistence boundary. Methods return futures so a database-backed store can replace the
/// in-memory one; `+ Send` lets axum move them between threads.
pub trait TodoStore: Send + Sync {
    fn list(&self) -> impl Future<Output = Vec<Todo>> + Send;
    fn get(&self, id: &str) -> impl Future<Output = Option<Todo>> + Send;
    fn save(&self, todo: Todo) -> impl Future<Output = ()> + Send;
    /// Delete; `true` if something was removed.
    fn delete(&self, id: &str) -> impl Future<Output = bool> + Send;
}

/// An error that knows which HTTP status it maps to. Implementing `IntoResponse` lets handlers
/// return `Result<_, ApiError>` and use `?`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    pub issues: Vec<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            issues: Vec::new(),
        }
    }

    pub fn not_found(id: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, format!("todo '{id}' not found"))
    }

    pub fn validation(issues: Vec<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "validation failed".to_string(),
            issues,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = if self.issues.is_empty() {
            json!({ "error": self.message })
        } else {
            json!({ "error": self.message, "issues": self.issues })
        };
        (self.status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests;
