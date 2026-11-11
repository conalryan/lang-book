import { printLine, readLines } from "./io.ts";
import { runSession } from "./logic.ts";

await runSession(readLines(), printLine);
