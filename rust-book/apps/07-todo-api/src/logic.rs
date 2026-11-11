//! Business rules and the in-memory store. Free of HTTP concerns apart from `ApiError`.

use std::collections::HashMap;

use chrono::{DateTime, SecondsFormat, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{ApiError, CreateTodo, Todo, TodoStore, UpdateTodo};

/// `HashMap` behind an async `Mutex`: many handlers share one store, and the lock guarantees
/// exclusive access while a handler mutates it.
#[derive(Debug, Default)]
pub struct InMemoryTodoStore {
    todos: Mutex<HashMap<String, Todo>>,
}

impl TodoStore for InMemoryTodoStore {
    async fn list(&self) -> Vec<Todo> {
        self.todos.lock().await.values().cloned().collect()
    }

    async fn get(&self, id: &str) -> Option<Todo> {
        self.todos.lock().await.get(id).cloned()
    }

    async fn save(&self, todo: Todo) {
        self.todos.lock().await.insert(todo.id.clone(), todo);
    }

    async fn delete(&self, id: &str) -> bool {
        self.todos.lock().await.remove(id).is_some()
    }
}

type IdFn = Box<dyn Fn() -> String + Send + Sync>;
type ClockFn = Box<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// Generic over the store: `S` is fixed at compile time, so there is no dynamic dispatch cost.
pub struct TodoService<S> {
    store: S,
    new_id: IdFn,
    now: ClockFn,
}

impl<S: TodoStore> TodoService<S> {
    /// A service with random UUID ids and the system clock.
    pub fn new(store: S) -> Self {
        Self {
            store,
            new_id: Box::new(|| Uuid::new_v4().to_string()),
            now: Box::new(Utc::now),
        }
    }

    /// Replace the id generator (tests use a counter). Builder style: takes and returns `self`.
    #[must_use]
    pub fn with_id_generator(mut self, f: impl Fn() -> String + Send + Sync + 'static) -> Self {
        self.new_id = Box::new(f);
        self
    }

    /// Replace the clock (tests use a fake).
    #[must_use]
    pub fn with_clock(mut self, f: impl Fn() -> DateTime<Utc> + Send + Sync + 'static) -> Self {
        self.now = Box::new(f);
        self
    }

    /// Todos, oldest first; optionally only those with the given `done` state.
    pub async fn list(&self, done: Option<bool>) -> Vec<Todo> {
        let mut todos: Vec<Todo> = self
            .store
            .list()
            .await
            .into_iter()
            .filter(|t| done.is_none_or(|d| t.done == d))
            .collect();
        todos.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        todos
    }

    /// One todo or a 404.
    pub async fn get(&self, id: &str) -> Result<Todo, ApiError> {
        self.store
            .get(id)
            .await
            .ok_or_else(|| ApiError::not_found(id))
    }

    /// Create an open todo.
    pub async fn create(&self, data: CreateTodo) -> Todo {
        let todo = Todo {
            id: (self.new_id)(),
            title: data.title,
            done: false,
            created_at: (self.now)().to_rfc3339_opts(SecondsFormat::Millis, true),
        };
        self.store.save(todo.clone()).await;
        todo
    }

    /// Apply the provided fields only.
    pub async fn update(&self, id: &str, patch: UpdateTodo) -> Result<Todo, ApiError> {
        let current = self.get(id).await?;
        let updated = Todo {
            title: patch.title.unwrap_or(current.title),
            done: patch.done.unwrap_or(current.done),
            ..current
        };
        self.store.save(updated.clone()).await;
        Ok(updated)
    }

    /// Delete or 404.
    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        if self.store.delete(id).await {
            Ok(())
        } else {
            Err(ApiError::not_found(id))
        }
    }
}
