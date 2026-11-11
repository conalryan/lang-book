import { expect, it } from "vitest";
import { tour } from "../src/tour.ts";

it("documents floating point and nullish behaviour", () => {
  const output = tour().join("\n");
  expect(output).toContain("0.30000000000000004");
  expect(output).toContain("18446744073709551616");
  expect(output).toMatch(/\?\? vs \|\|\s+0 5/);
});
