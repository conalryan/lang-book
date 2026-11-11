"""Process I/O: bind the app to a port."""

import uvicorn
from fastapi import FastAPI


def start_server(app: FastAPI, port: int) -> None:
    """Serve until Ctrl-C / SIGTERM; uvicorn handles graceful shutdown."""
    print(f"listening on http://localhost:{port}", flush=True)
    uvicorn.run(app, port=port, log_level="warning")
