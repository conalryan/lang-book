import { beforeEach, describe, expect, it } from "vitest";
import { createApp, InMemoryTodoStore, TodoService } from "../src/index.ts";

function setup() {
  let id = 0;
  let tick = 0;
  const service = new TodoService({
    store: new InMemoryTodoStore(),
    newId: () => `id-${++id}`,
    now: () => new Date(Date.UTC(2024, 0, 1, 0, 0, tick++)),
  });
  const app = createApp(service);

  const send = (path: string, method = "GET", body?: unknown) =>
    app.request(path, {
      method,
      ...(body === undefined
        ? {}
        : { body: typeof body === "string" ? body : JSON.stringify(body) }),
      headers: { "content-type": "application/json" },
    });
  return { send };
}

let send: ReturnType<typeof setup>["send"];

beforeEach(() => {
  ({ send } = setup());
});

describe("health and routing", () => {
  it("reports health", async () => {
    const res = await send("/health");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ok" });
  });

  it("returns a JSON 404 for unknown routes", async () => {
    const res = await send("/nope");
    expect(res.status).toBe(404);
    expect(await res.json()).toEqual({ error: "route not found" });
  });
});

describe("POST /todos", () => {
  it("creates a todo", async () => {
    const res = await send("/todos", "POST", { title: "  write docs  " });
    expect(res.status).toBe(201);
    expect(await res.json()).toEqual({
      id: "id-1",
      title: "write docs",
      done: false,
      createdAt: "2024-01-01T00:00:00.000Z",
    });
  });

  it.each([
    [{}, "title: "],
    [{ title: "" }, "title: "],
    [{ title: "x".repeat(201) }, "title: "],
    [{ title: "ok", extra: 1 }, "(body): "],
    [[], "(body): "],
  ])("rejects invalid body %j", async (body, prefix) => {
    const res = await send("/todos", "POST", body);
    expect(res.status).toBe(400);
    const json = (await res.json()) as { error: string; issues: string[] };
    expect(json.error).toBe("validation failed");
    expect(json.issues[0]).toContain(prefix);
  });

  it("rejects malformed JSON", async () => {
    const res = await send("/todos", "POST", "{not json");
    expect(res.status).toBe(400);
    expect(await res.json()).toEqual({ error: "invalid JSON body" });
  });
});

describe("GET /todos", () => {
  it("lists in creation order and filters by done", async () => {
    await send("/todos", "POST", { title: "a" });
    await send("/todos", "POST", { title: "b" });
    await send("/todos/id-1", "PATCH", { done: true });

    const all = (await (await send("/todos")).json()) as { title: string }[];
    expect(all.map((t) => t.title)).toEqual(["a", "b"]);

    const done = (await (await send("/todos?done=true")).json()) as { title: string }[];
    expect(done.map((t) => t.title)).toEqual(["a"]);

    const open = (await (await send("/todos?done=false")).json()) as { title: string }[];
    expect(open.map((t) => t.title)).toEqual(["b"]);
  });

  it("rejects a bad filter", async () => {
    expect((await send("/todos?done=maybe")).status).toBe(400);
  });
});

describe("GET/PATCH/DELETE /todos/:id", () => {
  it("gets one todo or 404s", async () => {
    await send("/todos", "POST", { title: "a" });
    expect((await send("/todos/id-1")).status).toBe(200);
    const missing = await send("/todos/zzz");
    expect(missing.status).toBe(404);
    expect(await missing.json()).toEqual({ error: "todo 'zzz' not found" });
  });

  it("patches only the provided fields", async () => {
    await send("/todos", "POST", { title: "a" });
    const res = await send("/todos/id-1", "PATCH", { done: true });
    expect(await res.json()).toMatchObject({ title: "a", done: true });
    const renamed = await send("/todos/id-1", "PATCH", { title: "b" });
    expect(await renamed.json()).toMatchObject({ title: "b", done: true });
  });

  it("rejects empty patches and unknown todos", async () => {
    await send("/todos", "POST", { title: "a" });
    expect((await send("/todos/id-1", "PATCH", {})).status).toBe(400);
    expect((await send("/todos/zzz", "PATCH", { done: true })).status).toBe(404);
  });

  it("deletes once", async () => {
    await send("/todos", "POST", { title: "a" });
    const first = await send("/todos/id-1", "DELETE");
    expect(first.status).toBe(204);
    expect(await first.text()).toBe("");
    expect((await send("/todos/id-1", "DELETE")).status).toBe(404);
  });
});
