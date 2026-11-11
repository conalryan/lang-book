import {
  type Clock,
  DEFAULT_POLICY,
  type Entity,
  err,
  type LibraryError,
  type LibraryItem,
  type Loan,
  type Member,
  ok,
  type Policy,
  type Receipt,
  type Repository,
  type Result,
} from "./models.ts";

const MS_PER_DAY = 86_400_000;

/** Generic class: one implementation serves items, members and loans. */
export class InMemoryRepository<T extends Entity> implements Repository<T> {
  readonly #store = new Map<string, T>();

  get(id: string): T | undefined {
    return this.#store.get(id);
  }

  add(item: T): void {
    this.#store.set(item.id, item);
  }

  remove(id: string): boolean {
    return this.#store.delete(id);
  }

  all(): T[] {
    return [...this.#store.values()];
  }

  find(predicate: (item: T) => boolean): T[] {
    return this.all().filter(predicate);
  }
}

/** A clock you can move forward by hand. */
export class ManualClock implements Clock {
  #current: Date;

  constructor(start: Date) {
    this.#current = start;
  }

  now(): Date {
    return this.#current;
  }

  advanceDays(days: number): void {
    this.#current = addDays(this.#current, days);
  }
}

export function addDays(date: Date, days: number): Date {
  return new Date(date.getTime() + days * MS_PER_DAY);
}

export function formatDate(date: Date): string {
  return date.toISOString().slice(0, 10);
}

export function formatMoney(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

export function describeError(error: LibraryError): string {
  switch (error.kind) {
    case "item-not-found":
      return `item '${error.id}' not found`;
    case "member-not-found":
      return `member '${error.id}' not found`;
    case "already-on-loan":
      return `'${error.title}' is already checked out`;
    case "not-on-loan":
      return `'${error.title}' is not checked out`;
    case "limit-reached":
      return `${error.name} has reached the limit of ${error.limit} loans`;
  }
}

/**
 * Composition: the service *has* repositories, a clock and a policy; it does not inherit from
 * any of them. Dependencies are injected through the constructor, which keeps it testable.
 */
export class LibraryService {
  readonly #items: Repository<LibraryItem>;
  readonly #members: Repository<Member>;
  readonly #loans: Repository<Loan>;
  readonly #clock: Clock;
  readonly #policy: Policy;

  constructor(
    items: Repository<LibraryItem>,
    members: Repository<Member>,
    loans: Repository<Loan>,
    clock: Clock,
    policy: Policy = DEFAULT_POLICY,
  ) {
    this.#items = items;
    this.#members = members;
    this.#loans = loans;
    this.#clock = clock;
    this.#policy = policy;
  }

  checkOut(
    itemId: string,
    memberId: string,
  ): Result<{ item: LibraryItem; loan: Loan; member: Member }, LibraryError> {
    const item = this.#items.get(itemId);
    if (item === undefined) return err({ kind: "item-not-found", id: itemId });
    const member = this.#members.get(memberId);
    if (member === undefined) return err({ kind: "member-not-found", id: memberId });
    if (this.#loans.get(itemId) !== undefined) {
      return err({ kind: "already-on-loan", title: item.title });
    }
    if (this.#loans.find((l) => l.memberId === memberId).length >= this.#policy.maxLoans) {
      return err({ kind: "limit-reached", name: member.name, limit: this.#policy.maxLoans });
    }

    // Polymorphism: loanDays comes from whichever subclass `item` really is.
    const loan: Loan = { id: itemId, memberId, dueOn: addDays(this.#clock.now(), item.loanDays) };
    this.#loans.add(loan);
    return ok({ item, loan, member });
  }

  returnItem(itemId: string): Result<Receipt, LibraryError> {
    const item = this.#items.get(itemId);
    if (item === undefined) return err({ kind: "item-not-found", id: itemId });
    const loan = this.#loans.get(itemId);
    if (loan === undefined) return err({ kind: "not-on-loan", title: item.title });

    const daysLate = Math.max(
      0,
      Math.ceil((this.#clock.now().getTime() - loan.dueOn.getTime()) / MS_PER_DAY),
    );
    this.#loans.remove(itemId);
    return ok({ title: item.title, daysLate, feeCents: daysLate * this.#policy.feePerDayCents });
  }
}
