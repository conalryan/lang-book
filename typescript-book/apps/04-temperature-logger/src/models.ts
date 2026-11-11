/**
 * Units as a const object + derived union. This is the erasable-syntax alternative to
 * `enum Unit { ... }` (real TS enums emit runtime code, so Node's type stripping rejects them).
 */
export const Unit = { Celsius: "C", Fahrenheit: "F", Kelvin: "K" } as const;
export type Unit = (typeof Unit)[keyof typeof Unit];

/** Result<T, E>: success or failure as a plain value. The TS spelling of Rust's Result. */
export type Result<T, E> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

export const ok = <T>(value: T): Result<T, never> => ({ ok: true, value });
export const err = <E>(error: E): Result<never, E> => ({ ok: false, error });

/** Tagged union: each variant carries different data, like a Rust enum. */
export type ParseError =
  | { readonly kind: "field-count"; readonly found: number }
  | { readonly kind: "bad-timestamp"; readonly text: string }
  | { readonly kind: "bad-value"; readonly text: string }
  | { readonly kind: "unknown-unit"; readonly text: string };

export interface Stats {
  readonly count: number;
  readonly min: number;
  readonly max: number;
  readonly mean: number;
  readonly median: number;
}

/** An immutable measurement. Value is stored as entered; `celsius` normalises it. */
export class Reading {
  readonly timestamp: string;
  readonly value: number;
  readonly unit: Unit;

  constructor(timestamp: string, value: number, unit: Unit) {
    this.timestamp = timestamp;
    this.value = value;
    this.unit = unit;
  }

  get celsius(): number {
    switch (this.unit) {
      case Unit.Celsius:
        return this.value;
      case Unit.Fahrenheit:
        return ((this.value - 32) * 5) / 9;
      case Unit.Kelvin:
        return this.value - 273.15;
      default: {
        const unreachable: never = this.unit;
        throw new Error(`unhandled unit ${String(unreachable)}`);
      }
    }
  }
}

/** Append-only collection of readings with derived statistics. */
export class TemperatureLog {
  // `#` fields are truly private at runtime (unlike TS `private`, which is compile-time only).
  readonly #readings: Reading[] = [];

  add(reading: Reading): this {
    this.#readings.push(reading);
    return this;
  }

  get size(): number {
    return this.#readings.length;
  }

  /** `undefined` when empty: callers must handle "no data" because the type says so. */
  latest(): Reading | undefined {
    return this.#readings.reduce<Reading | undefined>(
      (best, r) => (best === undefined || r.timestamp >= best.timestamp ? r : best),
      undefined,
    );
  }

  stats(): Stats | undefined {
    if (this.#readings.length === 0) return undefined;
    const values = this.#readings.map((r) => r.celsius).sort((a, b) => a - b);
    const count = values.length;
    const mid = Math.floor(count / 2);
    // noUncheckedIndexedAccess makes values[i] `number | undefined`; `as number` is safe here.
    const median =
      count % 2 === 1
        ? (values[mid] as number)
        : ((values[mid - 1] as number) + (values[mid] as number)) / 2;
    return {
      count,
      min: values[0] as number,
      max: values[count - 1] as number,
      mean: values.reduce((sum, v) => sum + v, 0) / count,
      median,
    };
  }
}
