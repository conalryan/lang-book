import { Hono } from "hono";
import type { z } from "zod";
import type { TodoService } from "./logic.ts";
import { CreateTodoSchema, HttpError, ListQuerySchema, UpdateTodoSchema } from "./models.ts";

function parseWith<T>(schema: z.ZodType<T>, data: unknown): T {
  const result = schema.safeParse(data);
  if (!result.success) {
    const issues = result.error.issues.map((i) => `${i.path.join(".") || "(body)"}: ${i.message}`);
    throw new HttpError(400, "validation failed", issues);
  }
  return result.data;
}

async function readJson(request: Request): Promise<unknown> {
  try {
    return await request.json();
  } catch {
    throw new HttpError(400, "invalid JSON body");
  }
}

/** Build the HTTP layer around a service. Routes only translate HTTP <-> service calls. */
export function createApp(service: TodoService): Hono {
  const app = new Hono();

  app.get("/health", (c) => c.json({ status: "ok" }));

  app.get("/todos", async (c) => {
    const { done } = parseWith(ListQuerySchema, c.req.query());
    return c.json(await service.list(done === undefined ? undefined : done === "true"));
  });

  app.post("/todos", async (c) => {
    const input = parseWith(CreateTodoSchema, await readJson(c.req.raw));
    return c.json(await service.create(input), 201);
  });

  app.get("/todos/:id", async (c) => c.json(await service.get(c.req.param("id"))));

  app.patch("/todos/:id", async (c) => {
    const patch = parseWith(UpdateTodoSchema, await readJson(c.req.raw));
    return c.json(await service.update(c.req.param("id"), patch));
  });

  app.delete("/todos/:id", async (c) => {
    await service.delete(c.req.param("id"));
    return c.body(null, 204);
  });

  app.notFound((c) => c.json({ error: "route not found" }, 404));

  app.onError((error, c) => {
    if (error instanceof HttpError) {
      const body =
        error.issues.length > 0
          ? { error: error.message, issues: error.issues }
          : { error: error.message };
      return c.json(body, error.status);
    }
    console.error(error);
    return c.json({ error: "internal error" }, 500);
  });

  return app;
}
