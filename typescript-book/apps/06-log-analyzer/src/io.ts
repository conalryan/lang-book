import { readFile } from "node:fs/promises";

export async function readFileLines(path: string): Promise<string[]> {
  return (await readFile(path, "utf8")).split(/\r?\n/);
}

export function printLine(text: string): void {
  process.stdout.write(`${text}\n`);
}

export function printError(text: string): void {
  process.stderr.write(`error: ${text}\n`);
}
