use std::process::ExitCode;

use todo_api::{InMemoryTodoStore, TodoService, create_app, serve};

/// `#[tokio::main]` turns `async fn main` into a normal `main` that starts the runtime.
#[tokio::main]
async fn main() -> ExitCode {
    let port = std::env::var("PORT").map_or(Ok(3000), |text| {
        text.parse::<u16>()
            .map_err(|_| format!("invalid PORT '{text}'"))
    });
    let port = match port {
        Ok(port) => port,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let app = create_app(TodoService::new(InMemoryTodoStore::default()));
    match serve(app, port).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
