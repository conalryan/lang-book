import { describe, expect, it } from "vitest";
import { compileFilter, evaluate } from "../src/evaluator.ts";
import { tokenize } from "../src/lexer.ts";
import {
  analyze,
  averageMs,
  countBy,
  filterLazily,
  formatReport,
  parseEntries,
  parseLine,
} from "../src/logic.ts";
import { FilterSyntaxError, type LogEntry } from "../src/models.ts";
import { parse } from "../src/parser.ts";

const SAMPLE = [
  "2024-03-01T10:00:01Z INFO api GET /users status=200 ms=12",
  "2024-03-01T10:00:02Z INFO api GET /users/7 status=200 ms=8",
  "2024-03-01T10:00:03Z WARN db slow query ms=950",
  "2024-03-01T10:00:04Z ERROR api POST /orders status=500 ms=1200",
  "2024-03-01T10:00:05Z DEBUG cache miss key=user:7",
  "this line is garbage",
  "",
  "2024-03-01T10:00:06Z ERROR db connection timeout ms=3000",
  "2024-03-01T10:00:07Z INFO web GET / status=200 ms=30",
];

function entry(line: string): LogEntry {
  const parsed = parseLine(line);
  if (parsed === undefined) throw new Error(`bad fixture: ${line}`);
  return parsed;
}

describe("parseLine", () => {
  it("extracts fields and key=value pairs", () => {
    const e = entry(SAMPLE[3] as string);
    expect(e).toMatchObject({
      level: "ERROR",
      service: "api",
      message: "POST /orders status=500 ms=1200",
    });
    expect(Object.fromEntries(e.fields)).toEqual({ status: "500", ms: "1200" });
  });

  it("returns undefined for garbage", () => {
    expect(parseLine("nope")).toBeUndefined();
    expect(parseLine("2024 TRACE api x")).toBeUndefined();
  });
});

describe("tokenize", () => {
  it("handles operators, strings, numbers and keywords", () => {
    const tokens = tokenize('status >= 500 AND (msg ~ "a \\"b\\"" or not x != -1.5)');
    expect(tokens.map((t) => t.type)).toEqual([
      "ident",
      "op",
      "number",
      "keyword",
      "paren",
      "ident",
      "op",
      "string",
      "keyword",
      "keyword",
      "ident",
      "op",
      "number",
      "paren",
      "end",
    ]);
    expect(tokens[7]).toMatchObject({ text: 'a "b"' });
  });

  it.each([
    ["status # 5", "unexpected character '#' at position 7"],
    ['msg ~ "open', "unterminated string at position 6"],
  ])("rejects %j", (source, message) => {
    expect(() => tokenize(source)).toThrow(message);
  });
});

describe("parse", () => {
  it("gives `and` higher precedence than `or`", () => {
    expect(parse("a = 1 or b = 2 and c = 3")).toMatchObject({
      type: "or",
      right: { type: "and" },
    });
  });

  it("respects parentheses and not", () => {
    expect(parse("not (a = 1 or b = 2)")).toMatchObject({ type: "not", operand: { type: "or" } });
  });

  it.each([
    ["", "expected a field name but found end of input"],
    ["level", "expected an operator but found end of input"],
    ["level =", "expected a value but found end of input"],
    ["(level = INFO", "expected ')' but found end of input"],
    ["level = INFO INFO", "unexpected 'INFO'"],
    ["= 1", "expected a field name but found '='"],
  ])("rejects %j", (source, message) => {
    expect(() => parse(source)).toThrow(FilterSyntaxError);
    expect(() => parse(source)).toThrow(message);
  });
});

describe("evaluate / compileFilter", () => {
  const entries = SAMPLE.map(parseLine).filter((e): e is LogEntry => e !== undefined);
  const run = (filter: string): string[] =>
    entries.filter(compileFilter(filter)).map((e) => `${e.service}:${e.level}`);

  it("compares levels by severity", () => {
    expect(run("level >= WARN")).toEqual(["db:WARN", "api:ERROR", "db:ERROR"]);
    expect(run("level < info")).toEqual(["cache:DEBUG"]);
    expect(run("level = error")).toEqual(["api:ERROR", "db:ERROR"]);
    expect(run("level = LOUD")).toEqual([]);
  });

  it("compares numeric fields numerically", () => {
    expect(run("ms > 100")).toEqual(["db:WARN", "api:ERROR", "db:ERROR"]);
    expect(run("status = 200 and ms < 10")).toEqual(["api:INFO"]);
  });

  it("supports substring, boolean logic and string compare", () => {
    expect(run('message ~ "timeout"')).toEqual(["db:ERROR"]);
    expect(run("service = api and not status = 200")).toEqual(["api:ERROR"]);
    expect(run("service = web or service = cache")).toEqual(["cache:DEBUG", "web:INFO"]);
    expect(run("service > cache and service <= db")).toEqual(["db:WARN", "db:ERROR"]);
    expect(run("service >= web")).toEqual(["web:INFO"]);
  });

  it("treats missing or non-numeric fields as non-matching, except for !=", () => {
    expect(run("status > 0")).toHaveLength(4);
    expect(run("status != 200")).toHaveLength(entries.length - 3);
    expect(run("key > 1")).toEqual([]);
    expect(run("key != 1")).toHaveLength(entries.length);
  });

  it("evaluate works directly on a tree", () => {
    expect(evaluate(parse("service = api"), entry(SAMPLE[0] as string))).toBe(true);
  });
});

describe("helpers", () => {
  it("parseEntries counts skipped lines but ignores blanks", () => {
    const { entries, skipped } = parseEntries(SAMPLE);
    expect(entries).toHaveLength(7);
    expect(skipped).toBe(1);
  });

  it("filterLazily does not run until iterated", () => {
    let calls = 0;
    const lazy = filterLazily([1, 2, 3], (n) => {
      calls += 1;
      return n > 1;
    });
    expect(calls).toBe(0);
    expect([...lazy]).toEqual([2, 3]);
    expect(calls).toBe(3);
  });

  it("countBy keeps first-seen order", () => {
    expect([...countBy(["b", "a", "b"], (s) => s)]).toEqual([
      ["b", 2],
      ["a", 1],
    ]);
  });

  it("averageMs ignores entries without ms", () => {
    expect(averageMs([entry(SAMPLE[0] as string), entry(SAMPLE[4] as string)])).toBe(12);
    expect(averageMs([entry(SAMPLE[4] as string)])).toBeUndefined();
  });
});

describe("analyze + formatReport", () => {
  it("summarises everything by default", () => {
    expect(formatReport(analyze(SAMPLE))).toEqual([
      "matched: 7 of 7",
      "levels: ERROR=2 WARN=1 INFO=3 DEBUG=1",
      "services: api=3 db=2 cache=1 web=1",
      "avg ms: 866.67",
      "skipped: 1",
    ]);
  });

  it("summarises a filtered view and can print matches", () => {
    expect(formatReport(analyze(SAMPLE, "level >= WARN"), { print: true })).toEqual([
      SAMPLE[2],
      SAMPLE[3],
      SAMPLE[7],
      "matched: 3 of 7",
      "levels: ERROR=2 WARN=1",
      "services: db=2 api=1",
      "avg ms: 1716.67",
      "skipped: 1",
    ]);
  });

  it("omits details when nothing matches", () => {
    expect(formatReport(analyze(SAMPLE, "service = nope"))).toEqual([
      "matched: 0 of 7",
      "skipped: 1",
    ]);
  });

  it("propagates filter syntax errors", () => {
    expect(() => analyze(SAMPLE, "level >")).toThrow(FilterSyntaxError);
  });
});
