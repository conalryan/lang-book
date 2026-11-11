import { printError, printLine } from "./io.ts";
import { execute, parseCommand } from "./logic.ts";

// Entry point: Node 24 runs this file directly (`node src/main.ts`) by stripping types.
try {
  printLine(execute(parseCommand(process.argv.slice(2))));
} catch (error) {
  // `error` is `unknown` in a catch clause; narrow before touching `.message`.
  printError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
