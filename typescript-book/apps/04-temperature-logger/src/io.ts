import { createInterface } from "node:readline";

/** Read all of stdin into an array of lines. */
export async function readAllLines(): Promise<string[]> {
  const lines: string[] = [];
  for await (const line of createInterface({ input: process.stdin })) {
    lines.push(line);
  }
  return lines;
}

export function printLine(text: string): void {
  process.stdout.write(`${text}\n`);
}
