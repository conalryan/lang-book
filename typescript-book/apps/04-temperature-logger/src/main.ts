import { printLine, readAllLines } from "./io.ts";
import { report } from "./logic.ts";

for (const line of report(await readAllLines())) {
  printLine(line);
}
