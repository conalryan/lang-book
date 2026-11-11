export type Result<T, E> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

export const ok = <T>(value: T): Result<T, never> => ({ ok: true, value });
export const err = <E>(error: E): Result<never, E> => ({ ok: false, error });

/** Anything stored in a repository needs a stable id. */
export interface Entity {
  readonly id: string;
}

/** Generic interface: `T` must satisfy `Entity` (a bound, like Rust's `T: Entity`). */
export interface Repository<T extends Entity> {
  get(id: string): T | undefined;
  add(item: T): void;
  remove(id: string): boolean;
  all(): T[];
  find(predicate: (item: T) => boolean): T[];
}

/**
 * Inheritance: Book and Dvd *are* LibraryItems and share id/title.
 * The abstract members force every subclass to define its own loan period.
 */
export abstract class LibraryItem implements Entity {
  readonly id: string;
  readonly title: string;
  abstract readonly loanDays: number;

  constructor(id: string, title: string) {
    this.id = id;
    this.title = title;
  }

  abstract describe(): string;
}

export class Book extends LibraryItem {
  readonly author: string;
  override readonly loanDays = 14;

  constructor(id: string, title: string, author: string) {
    super(id, title);
    this.author = author;
  }

  describe(): string {
    return `${this.title} by ${this.author}`;
  }
}

export class Dvd extends LibraryItem {
  readonly minutes: number;
  override readonly loanDays = 7;

  constructor(id: string, title: string, minutes: number) {
    super(id, title);
    this.minutes = minutes;
  }

  describe(): string {
    return `${this.title} (${this.minutes} min)`;
  }
}

/** Plain data: members are not polymorphic, so an interface is enough. */
export interface Member extends Entity {
  readonly name: string;
}

/** A loan is keyed by item id: one item can only be on loan once. */
export interface Loan extends Entity {
  readonly memberId: string;
  readonly dueOn: Date;
}

export interface Policy {
  readonly maxLoans: number;
  readonly feePerDayCents: number;
}

export const DEFAULT_POLICY: Policy = { maxLoans: 2, feePerDayCents: 25 };

/** Structural interface for time, so tests and demos control "now". */
export interface Clock {
  now(): Date;
}

export interface Receipt {
  readonly title: string;
  readonly daysLate: number;
  readonly feeCents: number;
}

export type LibraryError =
  | { readonly kind: "item-not-found"; readonly id: string }
  | { readonly kind: "member-not-found"; readonly id: string }
  | { readonly kind: "already-on-loan"; readonly title: string }
  | { readonly kind: "not-on-loan"; readonly title: string }
  | { readonly kind: "limit-reached"; readonly name: string; readonly limit: number };
