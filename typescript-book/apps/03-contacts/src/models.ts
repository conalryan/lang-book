export interface Contact {
  readonly name: string;
  readonly phone: string;
}

/** Keyed by lower-cased name so lookups are case-insensitive. Treated as immutable. */
export type ContactBook = ReadonlyMap<string, Contact>;

/** A tuple: fixed length, positional types. Every command returns [newBook, textToPrint]. */
export type CommandResult = readonly [book: ContactBook, output: string];

export const HELP = [
  "commands:",
  "  add <name> <phone>",
  "  remove <name>",
  "  find <text>",
  "  list",
  "  groups",
  "  count",
  "  help",
  "  quit",
].join("\n");
