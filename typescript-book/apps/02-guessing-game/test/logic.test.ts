import { describe, expect, it } from "vitest";
import {
  createRng,
  describeHint,
  intro,
  judge,
  parseInput,
  pickSecret,
  playGame,
  solve,
} from "../src/logic.ts";
import { DEFAULT_CONFIG } from "../src/models.ts";

const config = { min: 1, max: 100, maxAttempts: 3 };

async function play(secret: number, inputs: string[], cfg = config) {
  const output: string[] = [];
  const outcome = await playGame(secret, cfg, inputs, (line) => output.push(line));
  return { outcome, output };
}

describe("createRng / pickSecret", () => {
  it("is deterministic for a seed", () => {
    const a = createRng(42);
    const b = createRng(42);
    expect([a(), a(), a()]).toEqual([b(), b(), b()]);
  });

  it("stays inside the configured range", () => {
    const rng = createRng(7);
    for (let i = 0; i < 1000; i++) {
      const value = pickSecret(rng, DEFAULT_CONFIG);
      expect(value).toBeGreaterThanOrEqual(1);
      expect(value).toBeLessThanOrEqual(100);
    }
  });

  it("matches the cross-language reference value", () => {
    expect(pickSecret(createRng(1), DEFAULT_CONFIG)).toBe(63);
  });
});

describe("judge / describeHint", () => {
  it("compares guess to secret", () => {
    expect(judge(1, 5)).toBe("low");
    expect(judge(9, 5)).toBe("high");
    expect(judge(5, 5)).toBe("correct");
    expect([judge(1, 5), judge(9, 5), judge(5, 5)].map(describeHint)).toEqual([
      "Too low",
      "Too high",
      "Correct!",
    ]);
  });
});

describe("parseInput", () => {
  it.each([
    ["50", { kind: "guess", value: 50 }],
    [" 7 ", { kind: "guess", value: 7 }],
    ["Q", { kind: "quit" }],
    ["quit", { kind: "quit" }],
    ["0", { kind: "invalid" }],
    ["101", { kind: "invalid" }],
    ["-3", { kind: "invalid" }],
    ["abc", { kind: "invalid" }],
    ["", { kind: "invalid" }],
  ])("parses %j", (line, expected) => {
    expect(parseInput(line, config)).toEqual(expected);
  });
});

describe("playGame", () => {
  it("wins and counts attempts", async () => {
    const { outcome, output } = await play(30, ["10", "50", "30"]);
    expect(outcome).toEqual({ kind: "won", attempts: 3 });
    expect(output).toEqual(["Too low", "Too high", "Correct! You won in 3 attempts"]);
  });

  it("uses the singular for one attempt", async () => {
    const { output } = await play(30, ["30"]);
    expect(output).toEqual(["Correct! You won in 1 attempt"]);
  });

  it("does not charge attempts for invalid input", async () => {
    const { outcome, output } = await play(30, ["abc", "999", "30"]);
    expect(outcome).toEqual({ kind: "won", attempts: 1 });
    expect(output.slice(0, 2)).toEqual([
      "Please enter a whole number between 1 and 100",
      "Please enter a whole number between 1 and 100",
    ]);
  });

  it("loses after max attempts and ignores remaining input", async () => {
    const { outcome, output } = await play(30, ["1", "2", "3", "30"]);
    expect(outcome).toEqual({ kind: "lost", secret: 30 });
    expect(output.at(-1)).toBe("Out of attempts. The number was 30");
  });

  it("quits on request and when input ends", async () => {
    expect((await play(30, ["q"])).outcome).toEqual({ kind: "quit" });
    expect((await play(30, ["5"])).outcome).toEqual({ kind: "quit" });
  });

  it("accepts async iterables", async () => {
    async function* lines() {
      yield "30";
    }
    const output: string[] = [];
    await playGame(30, config, lines(), (l) => output.push(l));
    expect(output).toEqual(["Correct! You won in 1 attempt"]);
  });
});

describe("solve", () => {
  it("always finds the secret within ceil(log2(100)) = 7 attempts", () => {
    for (let secret = 1; secret <= 100; secret++) {
      expect(solve(secret, DEFAULT_CONFIG)).toBeLessThanOrEqual(7);
    }
  });
});

describe("intro", () => {
  it("describes the rules", () => {
    expect(intro(DEFAULT_CONFIG)).toBe(
      "Guess a number between 1 and 100. You have 10 attempts. Type 'q' to quit.",
    );
  });
});
