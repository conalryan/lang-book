import { describe, expect, it } from "vitest";
import { runDemo } from "../src/demo.ts";
import {
  addDays,
  describeError,
  formatDate,
  formatMoney,
  InMemoryRepository,
  LibraryService,
  ManualClock,
} from "../src/logic.ts";
import { Book, Dvd, type LibraryItem, type Loan, type Member } from "../src/models.ts";

function setup(maxLoans = 2) {
  const items = new InMemoryRepository<LibraryItem>();
  items.add(new Book("b1", "Dune", "Frank Herbert"));
  items.add(new Dvd("d1", "Alien", 117));
  items.add(new Book("b2", "Emma", "Jane Austen"));
  const members = new InMemoryRepository<Member>();
  members.add({ id: "m1", name: "Ada" });
  const clock = new ManualClock(new Date("2024-01-01T00:00:00Z"));
  const service = new LibraryService(items, members, new InMemoryRepository<Loan>(), clock, {
    maxLoans,
    feePerDayCents: 25,
  });
  return { service, clock };
}

describe("LibraryItem hierarchy", () => {
  it("shares fields and overrides behaviour", () => {
    const book = new Book("b", "Dune", "Frank Herbert");
    const dvd = new Dvd("d", "Alien", 117);
    expect(book.describe()).toBe("Dune by Frank Herbert");
    expect(dvd.describe()).toBe("Alien (117 min)");
    expect([book.loanDays, dvd.loanDays]).toEqual([14, 7]);
  });
});

describe("InMemoryRepository", () => {
  it("adds, finds, lists and removes", () => {
    const repo = new InMemoryRepository<Member>();
    repo.add({ id: "1", name: "Ada" });
    repo.add({ id: "2", name: "Grace" });
    expect(repo.get("1")?.name).toBe("Ada");
    expect(repo.find((m) => m.name.startsWith("G"))).toEqual([{ id: "2", name: "Grace" }]);
    expect(repo.all()).toHaveLength(2);
    expect(repo.remove("1")).toBe(true);
    expect(repo.remove("1")).toBe(false);
    expect(repo.get("1")).toBeUndefined();
  });
});

describe("date and money helpers", () => {
  it("formats", () => {
    expect(formatDate(addDays(new Date("2024-01-31T00:00:00Z"), 1))).toBe("2024-02-01");
    expect(formatMoney(75)).toBe("$0.75");
  });
});

describe("LibraryService.checkOut", () => {
  it("uses the item's own loan period", () => {
    const { service } = setup();
    const book = service.checkOut("b1", "m1");
    const dvd = service.checkOut("d1", "m1");
    expect(book.ok && formatDate(book.value.loan.dueOn)).toBe("2024-01-15");
    expect(dvd.ok && formatDate(dvd.value.loan.dueOn)).toBe("2024-01-08");
  });

  it.each([
    [["zzz", "m1"], { kind: "item-not-found", id: "zzz" }],
    [["b1", "zzz"], { kind: "member-not-found", id: "zzz" }],
  ])("fails for unknown ids %j", ([itemId, memberId], error) => {
    const { service } = setup();
    expect(service.checkOut(itemId as string, memberId as string)).toEqual({ ok: false, error });
  });

  it("rejects double checkout and enforces the limit", () => {
    const { service } = setup(1);
    service.checkOut("b1", "m1");
    expect(service.checkOut("b1", "m1")).toEqual({
      ok: false,
      error: { kind: "already-on-loan", title: "Dune" },
    });
    expect(service.checkOut("b2", "m1")).toEqual({
      ok: false,
      error: { kind: "limit-reached", name: "Ada", limit: 1 },
    });
  });
});

describe("LibraryService.returnItem", () => {
  it("charges per late day", () => {
    const { service, clock } = setup();
    service.checkOut("b1", "m1");
    clock.advanceDays(17);
    expect(service.returnItem("b1")).toEqual({
      ok: true,
      value: { title: "Dune", daysLate: 3, feeCents: 75 },
    });
  });

  it("is free when on time", () => {
    const { service, clock } = setup();
    service.checkOut("d1", "m1");
    clock.advanceDays(7);
    expect(service.returnItem("d1")).toEqual({
      ok: true,
      value: { title: "Alien", daysLate: 0, feeCents: 0 },
    });
  });

  it("fails for unknown or not-loaned items", () => {
    const { service } = setup();
    expect(service.returnItem("zzz")).toEqual({
      ok: false,
      error: { kind: "item-not-found", id: "zzz" },
    });
    expect(service.returnItem("b1")).toEqual({
      ok: false,
      error: { kind: "not-on-loan", title: "Dune" },
    });
  });
});

describe("describeError", () => {
  it("describes every variant", () => {
    expect(describeError({ kind: "item-not-found", id: "x" })).toBe("item 'x' not found");
    expect(describeError({ kind: "member-not-found", id: "x" })).toBe("member 'x' not found");
    expect(describeError({ kind: "not-on-loan", title: "T" })).toBe("'T' is not checked out");
  });
});

describe("runDemo", () => {
  it("prints the scripted scenario", () => {
    const lines: string[] = [];
    runDemo((l) => lines.push(l));
    expect(lines).toEqual([
      "Checked out 'Dune' to Ada, due 2024-01-15",
      "Checked out 'Alien' to Ada, due 2024-01-08",
      "error: Ada has reached the limit of 2 loans",
      "error: 'Dune' is already checked out",
      "Returned 'Alien' on time",
      "Returned 'Dune' 3 days late, fee $0.75",
      "error: 'Dune' is not checked out",
    ]);
  });
});
