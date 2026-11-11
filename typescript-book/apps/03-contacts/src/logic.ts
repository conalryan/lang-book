import { type CommandResult, type Contact, type ContactBook, HELP } from "./models.ts";

const keyOf = (name: string): string => name.toLowerCase();

const byName = (a: Contact, b: Contact): number => a.name.localeCompare(b.name, "en");

/** Returns a new book; the input is never mutated (spread copies the entries). */
export function addContact(book: ContactBook, contact: Contact): ContactBook | undefined {
  const key = keyOf(contact.name);
  if (book.has(key)) return undefined;
  return new Map([...book, [key, contact]]);
}

export function removeContact(book: ContactBook, name: string): ContactBook | undefined {
  const key = keyOf(name);
  if (!book.has(key)) return undefined;
  const next = new Map(book);
  next.delete(key);
  return next;
}

/** Default parameter: `limit` is optional and defaults to "no limit". */
export function findContacts(book: ContactBook, text: string, limit = Number.POSITIVE_INFINITY) {
  const needle = text.toLowerCase();
  return [...book.values()]
    .filter((c) => c.name.toLowerCase().includes(needle))
    .sort(byName)
    .slice(0, limit);
}

/** Split into [matching, rest] in one pass; returning a tuple avoids filtering twice. */
export function partition<T>(items: readonly T[], predicate: (item: T) => boolean): [T[], T[]] {
  const yes: T[] = [];
  const no: T[] = [];
  for (const item of items) {
    (predicate(item) ? yes : no).push(item);
  }
  return [yes, no];
}

/** Group names by upper-cased first letter: Map<"A", ["Adam", "Alice"]>, keys sorted. */
export function groupByInitial(book: ContactBook): Map<string, string[]> {
  const groups = new Map<string, string[]>();
  for (const { name } of [...book.values()].sort(byName)) {
    const initial = (name[0] ?? "?").toUpperCase();
    const names = groups.get(initial) ?? [];
    names.push(name);
    groups.set(initial, names);
  }
  return new Map([...groups].sort(([a], [b]) => a.localeCompare(b, "en")));
}

export function formatContact({ name, phone }: Contact): string {
  return `${name} ${phone}`;
}

/** Returns [commandWord, args]; `...rest` collects the remaining items into an array. */
export function splitCommand(line: string): [string, string[]] {
  const [command = "", ...args] = line.trim().split(/\s+/);
  return [command.toLowerCase(), args];
}

/** Interpret one line. Pure: takes a book, returns a new book plus the text to print. */
export function execute(book: ContactBook, line: string): CommandResult {
  const [command, args] = splitCommand(line);

  switch (command) {
    case "":
      return [book, ""];
    case "help":
      return [book, HELP];
    case "add": {
      const [name, phone, ...extra] = args;
      if (name === undefined || phone === undefined || extra.length > 0) {
        return [book, "error: usage: add <name> <phone>"];
      }
      const next = addContact(book, { name, phone });
      return next ? [next, `Added ${name}`] : [book, `error: ${name} already exists`];
    }
    case "remove": {
      const [name] = args;
      if (name === undefined) return [book, "error: usage: remove <name>"];
      const next = removeContact(book, name);
      return next ? [next, `Removed ${name}`] : [book, `error: ${name} not found`];
    }
    case "find": {
      const [text] = args;
      if (text === undefined) return [book, "error: usage: find <text>"];
      const matches = findContacts(book, text);
      return [book, matches.length === 0 ? "(no matches)" : matches.map(formatContact).join("\n")];
    }
    case "list": {
      const all = findContacts(book, "");
      return [book, all.length === 0 ? "(no contacts)" : all.map(formatContact).join("\n")];
    }
    case "groups": {
      const lines = [...groupByInitial(book)].map(
        ([initial, names]) => `${initial}: ${names.join(", ")}`,
      );
      return [book, lines.length === 0 ? "(no contacts)" : lines.join("\n")];
    }
    case "count":
      return [book, `${book.size} contact${book.size === 1 ? "" : "s"}`];
    default:
      return [book, `error: unknown command '${command}' (try 'help')`];
  }
}

/** Run lines until `quit` or end of input. Returns the final book. */
export async function runSession(
  lines: AsyncIterable<string> | Iterable<string>,
  write: (text: string) => void,
  initial: ContactBook = new Map(),
): Promise<ContactBook> {
  let book = initial;
  for await (const line of lines) {
    if (line.trim().toLowerCase() === "quit") {
      write("Bye");
      break;
    }
    const [next, output] = execute(book, line);
    book = next;
    if (output !== "") write(output);
  }
  return book;
}
