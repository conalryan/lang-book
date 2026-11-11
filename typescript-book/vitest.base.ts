import { defineConfig } from "vitest/config";

export const baseConfig = defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.ts"],
      // Entry points are thin wiring over tested modules.
      exclude: ["src/main.ts", "src/io.ts", "src/**/*.d.ts"],
      reporter: ["text", "html"],
    },
  },
});
