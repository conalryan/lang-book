/** Thin wrappers over the process streams so the rest of the code never touches globals. */
export function printLine(text: string): void {
  process.stdout.write(`${text}\n`);
}

export function printError(text: string): void {
  process.stderr.write(`error: ${text}\n`);
}
