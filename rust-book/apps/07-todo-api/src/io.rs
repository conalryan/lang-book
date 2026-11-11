//! Process I/O: bind the app to a port.

use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut stream) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            stream.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

/// Serve until Ctrl-C or SIGTERM, finishing in-flight requests first.
pub async fn serve(app: Router, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    println!(
        "listening on http://localhost:{}",
        listener.local_addr()?.port()
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}
