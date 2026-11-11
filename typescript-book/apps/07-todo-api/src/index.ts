/**
 * Public surface of the package. Anything not re-exported here is internal:
 * `package.json` "exports" stops consumers from deep-importing other files.
 */
export { createApp } from "./app.ts";
export { InMemoryTodoStore, TodoService, type TodoServiceDeps } from "./logic.ts";
export type { CreateTodo, Todo, TodoStore, UpdateTodo } from "./models.ts";
