/** Ordered by severity; the index doubles as the rank used by `level >= WARN`. */
export const LEVELS = ["DEBUG", "INFO", "WARN", "ERROR"] as const;
export type Level = (typeof LEVELS)[number];

export interface LogEntry {
  readonly raw: string;
  readonly timestamp: string;
  readonly level: Level;
  readonly service: string;
  readonly message: string;
  /** `key=value` pairs found in the message, e.g. status=500. */
  readonly fields: ReadonlyMap<string, string>;
}

export type CompareOp = "=" | "!=" | "<" | "<=" | ">" | ">=" | "~";

export type Token =
  | { readonly type: "ident" | "string" | "number"; readonly text: string; readonly pos: number }
  | { readonly type: "op"; readonly text: CompareOp; readonly pos: number }
  | { readonly type: "keyword"; readonly text: "and" | "or" | "not"; readonly pos: number }
  | { readonly type: "paren"; readonly text: "(" | ")"; readonly pos: number }
  | { readonly type: "end"; readonly text: ""; readonly pos: number };

/** Filter syntax tree. A recursive tagged union: nodes contain other nodes. */
export type Expr =
  | { readonly type: "and" | "or"; readonly left: Expr; readonly right: Expr }
  | { readonly type: "not"; readonly operand: Expr }
  | {
      readonly type: "compare";
      readonly field: string;
      readonly op: CompareOp;
      readonly value: string | number;
    };

export class FilterSyntaxError extends Error {
  readonly position: number;

  constructor(message: string, position: number) {
    super(`${message} at position ${position}`);
    this.name = "FilterSyntaxError";
    this.position = position;
  }
}

export interface Summary {
  readonly total: number;
  readonly skipped: number;
  readonly matched: readonly LogEntry[];
}
