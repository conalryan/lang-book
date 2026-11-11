import { createApp } from "./app.ts";
import { startServer } from "./io.ts";
import { InMemoryTodoStore, TodoService } from "./logic.ts";

const port = Number(process.env["PORT"] ?? 3000);
startServer(createApp(new TodoService({ store: new InMemoryTodoStore() })), port);
