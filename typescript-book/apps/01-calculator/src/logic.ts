import { type Command, OPERATORS, type Operator, UNIT_TABLE, type Unit } from "./models.ts";
import { tour } from "./tour.ts";

export const USAGE =
  "usage: calc <number> <op> <number> | calc convert <value> <from> <to> | calc tour";

/**
 * Parse a strict decimal number. `Number("")` is 0 and `Number("12px")` is NaN,
 * so we reject blanks explicitly. Throws on anything that is not finite.
 */
export function parseNumber(text: string): number {
  const value = text.trim() === "" ? Number.NaN : Number(text);
  if (!Number.isFinite(value)) {
    throw new Error(`'${text}' is not a number`);
  }
  return value;
}

/** Type guard: narrows `string` to `Operator` when it returns true. */
export function isOperator(text: string): text is Operator {
  return (OPERATORS as readonly string[]).includes(text);
}

export function isUnit(text: string): text is Unit {
  return Object.hasOwn(UNIT_TABLE, text);
}

/** Apply `op` to two numbers. `**` is exponentiation; `%` keeps the sign of the left operand. */
export function calculate(left: number, op: Operator, right: number): number {
  switch (op) {
    case "+":
      return left + right;
    case "-":
      return left - right;
    case "*":
      return left * right;
    case "/":
    case "%":
      if (right === 0) {
        throw new Error("division by zero");
      }
      return op === "/" ? left / right : left % right;
    case "**":
      return left ** right;
  }
}

/** Convert between two units of the same dimension via the base unit. */
export function convert(value: number, from: Unit, to: Unit): number {
  const source = UNIT_TABLE[from];
  const target = UNIT_TABLE[to];
  if (source.dimension !== target.dimension) {
    throw new Error(`cannot convert ${source.dimension} to ${target.dimension}`);
  }
  return target.fromBase(source.toBase(value));
}

/** Round to 6 decimals so 0.1 + 0.2 prints as 0.3 rather than 0.30000000000000004. */
export function formatNumber(value: number): string {
  return String(Number(value.toFixed(6)));
}

/** Turn raw argv (without node/script) into a typed command. */
export function parseCommand(args: readonly string[]): Command {
  const [first, second, third, ...rest] = args;

  if (first === "tour" && args.length === 1) {
    return { kind: "tour" };
  }

  if (first === "convert") {
    if (second === undefined || third === undefined || rest.length !== 1) {
      throw new Error(USAGE);
    }
    const [to] = rest as [string];
    if (!isUnit(third)) throw new Error(`unknown unit '${third}'`);
    if (!isUnit(to)) throw new Error(`unknown unit '${to}'`);
    return { kind: "convert", value: parseNumber(second), from: third, to };
  }

  if (args.length === 3 && first !== undefined && second !== undefined && third !== undefined) {
    if (!isOperator(second)) throw new Error(`unknown operator '${second}'`);
    return { kind: "calc", left: parseNumber(first), op: second, right: parseNumber(third) };
  }

  throw new Error(USAGE);
}

/** Run a command and return the exact text to print. */
export function execute(command: Command): string {
  switch (command.kind) {
    case "calc": {
      const { left, op, right } = command;
      return `${formatNumber(left)} ${op} ${formatNumber(right)} = ${formatNumber(calculate(left, op, right))}`;
    }
    case "convert": {
      const { value, from, to } = command;
      return `${formatNumber(value)} ${from} = ${formatNumber(convert(value, from, to))} ${to}`;
    }
    case "tour":
      return tour().join("\n");
  }
}
