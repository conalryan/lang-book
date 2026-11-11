import { serve } from "@hono/node-server";
import type { Hono } from "hono";

/** Bind the app to a port and shut down cleanly on Ctrl-C / SIGTERM. */
export function startServer(app: Hono, port: number): void {
  const server = serve({ fetch: app.fetch, port }, (info) => {
    process.stdout.write(`listening on http://localhost:${info.port}\n`);
  });
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.on(signal, () => server.close(() => process.exit(0)));
  }
}
