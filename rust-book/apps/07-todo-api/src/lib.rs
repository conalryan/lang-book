//! Capstone: a small REST API with axum.
//!
//! Visibility: the modules are private; only the items re-exported below form the public API.
//! Anything else is an implementation detail the compiler will not let consumers touch.

mod app;
mod io;
mod logic;
mod models;

pub use app::create_app;
pub use io::serve;
pub use logic::{InMemoryTodoStore, TodoService};
pub use models::{ApiError, CreateTodo, Todo, TodoStore, UpdateTodo};
