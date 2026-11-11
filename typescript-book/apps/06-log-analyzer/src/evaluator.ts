import { type CompareOp, type Expr, LEVELS, type LogEntry } from "./models.ts";
import { parse } from "./parser.ts";

function fieldValue(entry: LogEntry, field: string): string | undefined {
  switch (field) {
    case "level":
    case "service":
    case "message":
    case "timestamp":
      return entry[field];
    default:
      return entry.fields.get(field);
  }
}

/** Negative / zero / positive like `Array.sort`, or undefined when the values cannot be compared. */
function ordering(actual: string, expected: string | number, field: string): number | undefined {
  if (field === "level") {
    const a = LEVELS.indexOf(actual as (typeof LEVELS)[number]);
    const b = LEVELS.indexOf(String(expected).toUpperCase() as (typeof LEVELS)[number]);
    return a === -1 || b === -1 ? undefined : a - b;
  }
  if (typeof expected === "number") {
    const n = Number(actual);
    return Number.isNaN(n) ? undefined : n - expected;
  }
  return actual < expected ? -1 : actual > expected ? 1 : 0;
}

function compare(
  actual: string | undefined,
  op: CompareOp,
  expected: string | number,
  field: string,
): boolean {
  // A missing field only satisfies "!=".
  if (actual === undefined) return op === "!=";
  if (op === "~") return actual.includes(String(expected));

  const order = ordering(actual, expected, field);
  if (order === undefined) return op === "!=";

  switch (op) {
    case "=":
      return order === 0;
    case "!=":
      return order !== 0;
    case "<":
      return order < 0;
    case "<=":
      return order <= 0;
    case ">":
      return order > 0;
    case ">=":
      return order >= 0;
  }
}

/** Tree-walking interpreter: pattern match on the node type and recurse. */
export function evaluate(expr: Expr, entry: LogEntry): boolean {
  switch (expr.type) {
    case "and":
      return evaluate(expr.left, entry) && evaluate(expr.right, entry);
    case "or":
      return evaluate(expr.left, entry) || evaluate(expr.right, entry);
    case "not":
      return !evaluate(expr.operand, entry);
    case "compare":
      return compare(fieldValue(entry, expr.field), expr.op, expr.value, expr.field);
  }
}

/** Parse once, then return a closure that captures the tree and tests entries cheaply. */
export function compileFilter(source: string): (entry: LogEntry) => boolean {
  const tree = parse(source);
  return (entry) => evaluate(tree, entry);
}
