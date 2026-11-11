import { describe, expect, it } from "vitest";
import { buildLog, describeError, formatStats, parseReading, report } from "../src/logic.ts";
import { err, ok, Reading, TemperatureLog, Unit } from "../src/models.ts";

describe("Reading", () => {
  it("normalises to celsius", () => {
    expect(new Reading("t", 212, Unit.Fahrenheit).celsius).toBeCloseTo(100);
    expect(new Reading("t", 273.15, Unit.Kelvin).celsius).toBeCloseTo(0);
    expect(new Reading("t", 5, Unit.Celsius).celsius).toBe(5);
  });
});

describe("Result helpers", () => {
  it("builds tagged results", () => {
    expect(ok(1)).toEqual({ ok: true, value: 1 });
    expect(err("x")).toEqual({ ok: false, error: "x" });
  });
});

describe("parseReading", () => {
  it("parses a valid line (unit is case-insensitive)", () => {
    const result = parseReading("2024-01-01T08:00 21.5 c");
    expect(result).toEqual({ ok: true, value: new Reading("2024-01-01T08:00", 21.5, "C") });
  });

  it.each([
    ["only two", { kind: "field-count", found: 2 }],
    ["2024-13-45T99:99 1 C", { kind: "bad-timestamp", text: "2024-13-45T99:99" }],
    ["2024-02-30T10:00 1 C", { kind: "bad-timestamp", text: "2024-02-30T10:00" }],
    ["2024-01-01T24:00 1 C", { kind: "bad-timestamp", text: "2024-01-01T24:00" }],
    ["2024-01-01 1 C", { kind: "bad-timestamp", text: "2024-01-01" }],
    ["2024-01-01T08:00 hot C", { kind: "bad-value", text: "hot" }],
    ["2024-01-01T08:00 1 X", { kind: "unknown-unit", text: "X" }],
  ])("rejects %j", (line, error) => {
    expect(parseReading(line)).toEqual({ ok: false, error });
  });
});

describe("describeError", () => {
  it("describes every variant", () => {
    expect(describeError({ kind: "field-count", found: 1 })).toBe("expected 3 fields, found 1");
    expect(describeError({ kind: "bad-timestamp", text: "x" })).toBe("invalid timestamp 'x'");
    expect(describeError({ kind: "bad-value", text: "x" })).toBe("invalid temperature 'x'");
    expect(describeError({ kind: "unknown-unit", text: "x" })).toBe("unknown unit 'x'");
  });
});

describe("TemperatureLog", () => {
  it("is empty by default", () => {
    const log = new TemperatureLog();
    expect(log.size).toBe(0);
    expect(log.stats()).toBeUndefined();
    expect(log.latest()).toBeUndefined();
  });

  it("computes odd and even medians", () => {
    const log = new TemperatureLog()
      .add(new Reading("a", 1, "C"))
      .add(new Reading("b", 9, "C"))
      .add(new Reading("c", 5, "C"));
    expect(log.stats()).toMatchObject({ count: 3, min: 1, max: 9, mean: 5, median: 5 });
    log.add(new Reading("d", 7, "C"));
    expect(log.stats()?.median).toBe(6);
  });

  it("finds the latest by timestamp, not insertion order", () => {
    const log = new TemperatureLog()
      .add(new Reading("2024-02", 1, "C"))
      .add(new Reading("2024-01", 2, "C"));
    expect(log.latest()?.timestamp).toBe("2024-02");
  });
});

describe("buildLog / report", () => {
  it("collects problems with line numbers and skips blanks", () => {
    const { log, problems } = buildLog(["2024-01-01T08:00 1 C", "", "bad"]);
    expect(log.size).toBe(1);
    expect(problems).toEqual(["line 3: expected 3 fields, found 1"]);
  });

  it("formats an empty report", () => {
    expect(report([])).toEqual(["count: 0"]);
    expect(formatStats(undefined, undefined)).toEqual(["count: 0"]);
  });

  it("produces the full report", () => {
    expect(
      report([
        "2024-01-01T08:00 18.5 C",
        "2024-01-01T12:00 24 C",
        "2024-01-01T16:00 70 F",
        "oops",
        "2024-01-01T20:00 abc C",
        "2024-01-01T21:00 295.15 K",
      ]),
    ).toEqual([
      "line 4: expected 3 fields, found 1",
      "line 5: invalid temperature 'abc'",
      "count: 4",
      "min: 18.5 C",
      "max: 24 C",
      "mean: 21.4 C",
      "median: 21.56 C",
      "latest: 2024-01-01T21:00 22 C",
    ]);
  });
});
