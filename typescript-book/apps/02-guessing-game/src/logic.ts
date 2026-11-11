import type { GameConfig, GameOutcome, Hint, PlayerInput } from "./models.ts";

/**
 * Deterministic 32-bit PRNG (mulberry32). Returns a function: a closure that
 * captures and mutates `state` between calls. Same seed => same sequence in any language.
 */
export function createRng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Pick an integer in [min, max] inclusive. */
export function pickSecret(rng: () => number, { min, max }: GameConfig): number {
  return min + Math.floor(rng() * (max - min + 1));
}

export function judge(guess: number, secret: number): Hint {
  if (guess < secret) return "low";
  if (guess > secret) return "high";
  return "correct";
}

export function describeHint(hint: Hint): string {
  switch (hint) {
    case "low":
      return "Too low";
    case "high":
      return "Too high";
    case "correct":
      return "Correct!";
  }
}

/** `q`/`quit` ends the game; digits in range are guesses; everything else is invalid. */
export function parseInput(line: string, { min, max }: GameConfig): PlayerInput {
  const text = line.trim().toLowerCase();
  if (text === "q" || text === "quit") {
    return { kind: "quit" };
  }
  if (!/^\d+$/.test(text)) {
    return { kind: "invalid" };
  }
  const value = Number.parseInt(text, 10);
  return value >= min && value <= max ? { kind: "guess", value } : { kind: "invalid" };
}

export function intro({ min, max, maxAttempts }: GameConfig): string {
  return `Guess a number between ${min} and ${max}. You have ${maxAttempts} attempts. Type 'q' to quit.`;
}

/**
 * Drive one game. `lines` can be sync or async (`for await` accepts both), so tests
 * can pass an array and the CLI can pass stdin. Invalid input does not use up an attempt.
 */
export async function playGame(
  secret: number,
  config: GameConfig,
  lines: AsyncIterable<string> | Iterable<string>,
  write: (text: string) => void,
): Promise<GameOutcome> {
  let attempts = 0;

  for await (const line of lines) {
    const input = parseInput(line, config);

    if (input.kind === "quit") {
      write("Bye");
      return { kind: "quit" };
    }
    if (input.kind === "invalid") {
      write(`Please enter a whole number between ${config.min} and ${config.max}`);
      continue;
    }

    attempts += 1;
    const hint = judge(input.value, secret);
    if (hint === "correct") {
      write(`Correct! You won in ${attempts} attempt${attempts === 1 ? "" : "s"}`);
      return { kind: "won", attempts };
    }

    write(describeHint(hint));
    if (attempts >= config.maxAttempts) {
      break;
    }
  }

  // Reached by running out of attempts, or by input ending before the game did.
  if (attempts >= config.maxAttempts) {
    write(`Out of attempts. The number was ${secret}`);
    return { kind: "lost", secret };
  }
  write("Bye");
  return { kind: "quit" };
}

/** Binary-search bot: a `while` loop that halves the range each round. Returns attempts used. */
export function solve(secret: number, { min, max }: GameConfig): number {
  let low = min;
  let high = max;
  let attempts = 0;

  while (low <= high) {
    const guess = Math.floor((low + high) / 2);
    attempts += 1;
    const hint = judge(guess, secret);
    if (hint === "correct") break;
    if (hint === "low") low = guess + 1;
    else high = guess - 1;
  }
  return attempts;
}
