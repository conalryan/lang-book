import { z } from "zod";

const title = z.string().trim().min(1).max(200);

/**
 * Zod schemas are the single source of truth: they validate at runtime and
 * `z.infer` derives the static type, so the two can never drift apart.
 */
export const CreateTodoSchema = z.strictObject({ title });

export const UpdateTodoSchema = z
  .strictObject({ title: title.optional(), done: z.boolean().optional() })
  .refine((patch) => patch.title !== undefined || patch.done !== undefined, {
    message: "provide at least one of: title, done",
  });

export const ListQuerySchema = z.object({ done: z.enum(["true", "false"]).optional() });

export type CreateTodo = z.infer<typeof CreateTodoSchema>;
export type UpdateTodo = z.infer<typeof UpdateTodoSchema>;

export interface Todo {
  readonly id: string;
  readonly title: string;
  readonly done: boolean;
  readonly createdAt: string;
}

/** Persistence boundary. Async so a database-backed store can replace the in-memory one. */
export interface TodoStore {
  list(): Promise<Todo[]>;
  get(id: string): Promise<Todo | undefined>;
  save(todo: Todo): Promise<void>;
  delete(id: string): Promise<boolean>;
}

/** An error that knows which HTTP status it maps to. */
export class HttpError extends Error {
  readonly status: 400 | 404;
  readonly issues: readonly string[];

  constructor(status: 400 | 404, message: string, issues: readonly string[] = []) {
    super(message);
    this.name = "HttpError";
    this.status = status;
    this.issues = issues;
  }
}
