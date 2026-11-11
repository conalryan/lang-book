import { createInterface } from "node:readline";

/** stdin as an async iterable of lines; works interactively and when piped. */
export function readLines(): AsyncIterable<string> {
  return createInterface({ input: process.stdin });
}

export function printLine(text: string): void {
  process.stdout.write(`${text}\n`);
}
