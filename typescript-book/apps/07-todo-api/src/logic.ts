import {
  type CreateTodo,
  HttpError,
  type Todo,
  type TodoStore,
  type UpdateTodo,
} from "./models.ts";

export class InMemoryTodoStore implements TodoStore {
  readonly #todos = new Map<string, Todo>();

  async list(): Promise<Todo[]> {
    return [...this.#todos.values()];
  }

  async get(id: string): Promise<Todo | undefined> {
    return this.#todos.get(id);
  }

  async save(todo: Todo): Promise<void> {
    this.#todos.set(todo.id, todo);
  }

  async delete(id: string): Promise<boolean> {
    return this.#todos.delete(id);
  }
}

/** Collaborators are injected so tests can fix ids and time. */
export interface TodoServiceDeps {
  readonly store: TodoStore;
  readonly newId?: () => string;
  readonly now?: () => Date;
}

/** Business rules live here, free of HTTP concerns apart from the error type. */
export class TodoService {
  readonly #store: TodoStore;
  readonly #newId: () => string;
  readonly #now: () => Date;

  constructor({
    store,
    newId = () => crypto.randomUUID(),
    now = () => new Date(),
  }: TodoServiceDeps) {
    this.#store = store;
    this.#newId = newId;
    this.#now = now;
  }

  async list(done?: boolean): Promise<Todo[]> {
    const todos = await this.#store.list();
    return todos
      .filter((todo) => done === undefined || todo.done === done)
      .sort((a, b) => a.createdAt.localeCompare(b.createdAt) || a.id.localeCompare(b.id));
  }

  async get(id: string): Promise<Todo> {
    const todo = await this.#store.get(id);
    if (todo === undefined) throw new HttpError(404, `todo '${id}' not found`);
    return todo;
  }

  async create({ title }: CreateTodo): Promise<Todo> {
    const todo: Todo = {
      id: this.#newId(),
      title,
      done: false,
      createdAt: this.#now().toISOString(),
    };
    await this.#store.save(todo);
    return todo;
  }

  async update(id: string, patch: UpdateTodo): Promise<Todo> {
    const current = await this.get(id);
    // `??` rather than spread: with exactOptionalPropertyTypes an explicit undefined must not overwrite.
    const next: Todo = {
      ...current,
      title: patch.title ?? current.title,
      done: patch.done ?? current.done,
    };
    await this.#store.save(next);
    return next;
  }

  async delete(id: string): Promise<void> {
    if (!(await this.#store.delete(id))) throw new HttpError(404, `todo '${id}' not found`);
  }
}
