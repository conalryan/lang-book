import { describe, expect, it } from "vitest";
import {
  addContact,
  execute,
  findContacts,
  groupByInitial,
  partition,
  removeContact,
  runSession,
  splitCommand,
} from "../src/logic.ts";
import type { ContactBook } from "../src/models.ts";

const empty: ContactBook = new Map();

function seeded(): ContactBook {
  let book = empty;
  for (const [name, phone] of [
    ["Alice", "555-0001"],
    ["adam", "555-0002"],
    ["Bob", "555-0003"],
  ] as const) {
    book = addContact(book, { name, phone }) ?? book;
  }
  return book;
}

describe("addContact / removeContact", () => {
  it("does not mutate the original book", () => {
    const next = addContact(empty, { name: "Alice", phone: "1" });
    expect(empty.size).toBe(0);
    expect(next?.size).toBe(1);
  });

  it("rejects duplicates case-insensitively", () => {
    expect(addContact(seeded(), { name: "ALICE", phone: "9" })).toBeUndefined();
  });

  it("removes and reports missing", () => {
    expect(removeContact(seeded(), "BOB")?.size).toBe(2);
    expect(removeContact(seeded(), "zed")).toBeUndefined();
  });
});

describe("findContacts", () => {
  it("matches substrings, sorted, with optional limit", () => {
    expect(findContacts(seeded(), "a").map((c) => c.name)).toEqual(["adam", "Alice"]);
    expect(findContacts(seeded(), "a", 1)).toHaveLength(1);
  });
});

describe("partition", () => {
  it("returns a [matching, rest] tuple", () => {
    const [evens, odds] = partition([1, 2, 3, 4, 5], (n) => n % 2 === 0);
    expect(evens).toEqual([2, 4]);
    expect(odds).toEqual([1, 3, 5]);
  });
});

describe("groupByInitial", () => {
  it("groups and sorts", () => {
    expect([...groupByInitial(seeded())]).toEqual([
      ["A", ["adam", "Alice"]],
      ["B", ["Bob"]],
    ]);
  });
});

describe("splitCommand", () => {
  it("splits on whitespace and lower-cases the command", () => {
    expect(splitCommand("  ADD  Alice   555 ")).toEqual(["add", ["Alice", "555"]]);
    expect(splitCommand("")).toEqual(["", []]);
  });
});

describe("execute", () => {
  it("lists sorted contacts", () => {
    expect(execute(seeded(), "list")[1]).toBe("adam 555-0002\nAlice 555-0001\nBob 555-0003");
  });

  it.each([
    ["add Zed 1", "Added Zed"],
    ["add Alice 1", "error: Alice already exists"],
    ["add Alice", "error: usage: add <name> <phone>"],
    ["remove Bob", "Removed Bob"],
    ["remove Zed", "error: Zed not found"],
    ["remove", "error: usage: remove <name>"],
    ["find bo", "Bob 555-0003"],
    ["find zzz", "(no matches)"],
    ["find", "error: usage: find <text>"],
    ["groups", "A: adam, Alice\nB: Bob"],
    ["count", "3 contacts"],
    ["", ""],
    ["dance", "error: unknown command 'dance' (try 'help')"],
  ])("%j -> %j", (line, expected) => {
    expect(execute(seeded(), line)[1]).toBe(expected);
  });

  it("handles empty books", () => {
    expect(execute(empty, "list")[1]).toBe("(no contacts)");
    expect(execute(empty, "groups")[1]).toBe("(no contacts)");
    expect(execute(empty, "count")[1]).toBe("0 contacts");
  });

  it("prints help", () => {
    expect(execute(empty, "help")[1]).toContain("commands:");
  });
});

describe("runSession", () => {
  it("runs until quit and returns the final book", async () => {
    const out: string[] = [];
    const book = await runSession(["add Alice 1", "", "list", "quit", "add Never 2"], (t) =>
      out.push(t),
    );
    expect(out).toEqual(["Added Alice", "Alice 1", "Bye"]);
    expect(book.size).toBe(1);
  });
});
