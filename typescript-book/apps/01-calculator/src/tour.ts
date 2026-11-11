/** A guided tour of primitive types and variable declarations. Returns lines so it is testable. */
export function tour(): string[] {
  const lines: string[] = [];
  const say = (label: string, value: unknown): void => {
    lines.push(`${label.padEnd(18)}${String(value)}`);
  };

  // `const` cannot be reassigned; `let` can. Prefer `const`; there is no `var` in modern code.
  const answer = 42;
  let counter = 0;
  counter += 1; // compound assignment: += -= *= /= %= **= ??= ||= &&=

  // number is a 64-bit float for both integers and decimals.
  say("number", answer);
  say("0.1 + 0.2", 0.1 + 0.2);
  say("1_000_000", 1_000_000); // numeric separators are purely visual
  say("7 / 2", 7 / 2);
  say("trunc(7 / 2)", Math.trunc(7 / 2)); // there is no integer division operator
  say("-7 % 3", -7 % 3); // remainder keeps the sign of the left side
  say("isNaN(NaN)", Number.isNaN(Number.NaN)); // NaN !== NaN, so never compare against it

  // bigint is arbitrary precision and never mixes implicitly with number.
  say("bigint", 2n ** 64n);
  say("MAX_SAFE_INTEGER", Number.MAX_SAFE_INTEGER);

  // string is immutable UTF-16; template literals interpolate.
  const name = "TypeScript";
  say("template", `Hello, ${name}! counter=${counter}`);
  say("length", name.length);

  // boolean, plus the two "nothing" values: null (explicit) and undefined (absent).
  say("boolean", answer > 10 && !(answer < 0));
  say("null", null);
  say("undefined", undefined);
  say("typeof null", typeof null); // historical quirk: "object"
  const zero = Math.trunc(0.5); // typed as plain number, so ?? / || are meaningful below
  say("?? vs ||", `${zero ?? 5} ${zero || 5}`); // ?? only replaces null/undefined

  // symbol is a unique, non-string key.
  say("symbol", Symbol("id").description);

  // === never coerces. Avoid ==.
  say("'1' === 1", "1" === (1 as unknown));

  return lines;
}
