import { parseArgs } from "node:util";
import { printError, printLine, readFileLines } from "./io.ts";
import { analyze, formatReport } from "./logic.ts";

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    filter: { type: "string", short: "f" },
    print: { type: "boolean", short: "p", default: false },
  },
});

const [path] = positionals;

try {
  if (path === undefined) {
    throw new Error("usage: log-analyzer <file> [--filter <expr>] [--print]");
  }
  const lines = await readFileLines(path);
  for (const line of formatReport(analyze(lines, values.filter), { print: values.print })) {
    printLine(line);
  }
} catch (error) {
  printError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
