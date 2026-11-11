/** Binary operators the calculator understands. `as const` keeps the literal types. */
export const OPERATORS = ["+", "-", "*", "/", "%", "**"] as const;

/** Union of the literal strings above: "+" | "-" | "*" | "/" | "%" | "**". */
export type Operator = (typeof OPERATORS)[number];

export type Unit = "km" | "mi" | "kg" | "lb" | "c" | "f";

export type Dimension = "length" | "mass" | "temperature";

/** Every unit knows how to convert to and from a base unit (km, kg, celsius). */
export interface UnitInfo {
  readonly dimension: Dimension;
  toBase(value: number): number;
  fromBase(value: number): number;
}

export const UNIT_TABLE: Readonly<Record<Unit, UnitInfo>> = {
  km: { dimension: "length", toBase: (v) => v, fromBase: (v) => v },
  mi: { dimension: "length", toBase: (v) => v * 1.609344, fromBase: (v) => v / 1.609344 },
  kg: { dimension: "mass", toBase: (v) => v, fromBase: (v) => v },
  lb: { dimension: "mass", toBase: (v) => v * 0.45359237, fromBase: (v) => v / 0.45359237 },
  c: { dimension: "temperature", toBase: (v) => v, fromBase: (v) => v },
  f: {
    dimension: "temperature",
    toBase: (v) => ((v - 32) * 5) / 9,
    fromBase: (v) => (v * 9) / 5 + 32,
  },
};

/**
 * A parsed command line. Discriminated union: `kind` tells the compiler which
 * other fields exist (the closest TS gets to Rust's data-carrying enums).
 */
export type Command =
  | { readonly kind: "calc"; readonly left: number; readonly op: Operator; readonly right: number }
  | { readonly kind: "convert"; readonly value: number; readonly from: Unit; readonly to: Unit }
  | { readonly kind: "tour" };
