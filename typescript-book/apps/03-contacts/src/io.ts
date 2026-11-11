import { createInterface } from "node:readline";

export function readLines(): AsyncIterable<string> {
  return createInterface({ input: process.stdin });
}

export function printLine(text: string): void {
  process.stdout.write(`${text}\n`);
}
