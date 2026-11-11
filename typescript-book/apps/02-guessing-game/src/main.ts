import { printLine, readLines } from "./io.ts";
import { createRng, intro, pickSecret, playGame, solve } from "./logic.ts";
import { DEFAULT_CONFIG } from "./models.ts";

// GUESS_SEED makes the secret reproducible (same value => same number in every language port).
const seed = Number(process.env["GUESS_SEED"] ?? Date.now());
const secret = pickSecret(createRng(seed), DEFAULT_CONFIG);

if (process.argv.includes("--auto")) {
  const attempts = solve(secret, DEFAULT_CONFIG);
  printLine(`Bot found ${secret} in ${attempts} attempts`);
} else {
  printLine(intro(DEFAULT_CONFIG));
  await playGame(secret, DEFAULT_CONFIG, readLines(), printLine);
}
