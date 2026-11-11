import { describe, expect, it } from "vitest";
import {
  calculate,
  convert,
  execute,
  formatNumber,
  isOperator,
  parseCommand,
  parseNumber,
} from "../src/logic.ts";

describe("parseNumber", () => {
  it("parses decimals and negatives", () => {
    expect(parseNumber("-2.5")).toBe(-2.5);
  });

  it.each(["", "  ", "abc", "12px", "Infinity"])("rejects %j", (text) => {
    expect(() => parseNumber(text)).toThrow("is not a number");
  });
});

describe("calculate", () => {
  it.each([
    [3, "+", 4, 7],
    [3, "-", 4, -1],
    [3, "*", 4, 12],
    [9, "/", 2, 4.5],
    [-7, "%", 3, -1],
    [2, "**", 10, 1024],
  ] as const)("%d %s %d = %d", (left, op, right, expected) => {
    expect(calculate(left, op, right)).toBe(expected);
  });

  it("rejects division by zero", () => {
    expect(() => calculate(1, "/", 0)).toThrow("division by zero");
    expect(() => calculate(1, "%", 0)).toThrow("division by zero");
  });
});

describe("convert", () => {
  it("converts length, mass and temperature", () => {
    expect(formatNumber(convert(1, "mi", "km"))).toBe("1.609344");
    expect(formatNumber(convert(10, "kg", "lb"))).toBe("22.046226");
    expect(formatNumber(convert(100, "c", "f"))).toBe("212");
    expect(formatNumber(convert(-40, "f", "c"))).toBe("-40");
  });

  it("rejects mismatched dimensions", () => {
    expect(() => convert(1, "km", "kg")).toThrow("cannot convert length to mass");
  });
});

describe("formatNumber", () => {
  it("hides floating point noise", () => {
    expect(formatNumber(0.1 + 0.2)).toBe("0.3");
  });
});

describe("isOperator", () => {
  it("accepts known operators only", () => {
    expect(isOperator("**")).toBe(true);
    expect(isOperator("^")).toBe(false);
  });
});

describe("parseCommand + execute", () => {
  it("runs a calculation", () => {
    expect(execute(parseCommand(["3", "+", "4"]))).toBe("3 + 4 = 7");
  });

  it("runs a conversion", () => {
    expect(execute(parseCommand(["convert", "100", "c", "f"]))).toBe("100 c = 212 f");
  });

  it("runs the tour", () => {
    expect(execute(parseCommand(["tour"]))).toContain("0.1 + 0.2");
  });

  it.each([
    [["1", "^", "2"], "unknown operator '^'"],
    [["x", "+", "2"], "'x' is not a number"],
    [["convert", "1", "km", "parsec"], "unknown unit 'parsec'"],
    [["convert", "1", "km"], "usage:"],
    [[], "usage:"],
  ])("rejects %j", (args, message) => {
    expect(() => parseCommand(args)).toThrow(message);
  });
});
