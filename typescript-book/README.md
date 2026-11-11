# TypeScript Book

Seven small, runnable apps that walk through TypeScript one concept cluster at a time. Each app has the same layout and the same CLI behaviour as its Python and Rust counterparts, so the code can be diffed across languages.

## Apps

| # | App | New concepts |
|---|-----|--------------|
| 1 | [calculator](./apps/01-calculator) | entry point, primitives, variables, operators, parsing, `const`/`let` |
| 2 | [guessing-game](./apps/02-guessing-game) | `if`/`switch`/loops, `break`/`continue`, closures, async iteration |
| 3 | [contacts](./apps/03-contacts) | arrays, `Map`, functions, tuples, destructuring, spread/rest |
| 4 | [temperature-logger](./apps/04-temperature-logger) | classes, enums, optionality, `Result`, tagged unions |
| 5 | [library](./apps/05-library) | interfaces, inheritance vs composition, generics, DI |
| 6 | [log-analyzer](./apps/06-log-analyzer) | regex, generators, closures, recursive-descent parser, modules |
| 7 | [todo-api](./apps/07-todo-api) | dependencies, async, HTTP, schema validation, public API surface, tests |

## Toolchain

| Concern | Tool |
|---------|------|
| Language version | Node 24 (`.node-version`; works with fnm, mise, nvm, Volta) |
| Language | TypeScript 7 (native compiler), strict mode, `erasableSyntaxOnly` |
| Package manager | pnpm 12 workspace with a dependency catalog (`pnpm-workspace.yaml`) |
| Task runner + build cache | Turborepo (`turbo.json`) |
| Tests + coverage | Vitest + V8 coverage |
| Format + lint | Biome (`biome.json`) |
| Typecheck | `tsc` (per app) |

Node 24 runs `.ts` files directly by stripping types, so there is no dev-time build step: `node src/main.ts`. `pnpm build` still emits `dist/` with `tsc` to prove the production path.

## Getting started

```sh
corepack enable          # provides the pinned pnpm version
pnpm install

pnpm verify              # lint + typecheck + test (with coverage) + build, all cached
pnpm test                # tests for every app
pnpm lint:fix            # format + autofix

pnpm --filter @typescript-book/calculator start 3 + 4
```

## Layout

```text
apps/NN-name/
  src/
    models.ts     types, interfaces, classes, constants (the data)
    logic.ts      pure functions and services (the behaviour)
    io.ts         the only file that touches stdin/stdout/files/network
    main.ts       entry point: wiring only
  test/           Vitest tests
  package.json    scripts: start, build, typecheck, test
  tsconfig.json   extends ../../tsconfig.base.json (typecheck src + test)
  tsconfig.build.json   emits dist/ from src only
```

Why split this way: `logic.ts` has no I/O, so it is trivially testable and ports one-to-one to Rust or Python modules. `main.ts` and `io.ts` are excluded from coverage because they are thin wiring. Apps 5 to 7 add files (`demo.ts`, lexer/parser/evaluator, `app.ts`) when one concept deserves its own module.

## Cross-language conventions

- Same commands, same messages, same exit codes. A seed (`GUESS_SEED`) and fixed clocks make output deterministic.
- Comments only where TypeScript behaves differently from what you would expect.
- Errors that callers are expected to handle are values (`Result`, `T | undefined`); `throw` is for programmer mistakes and I/O failures.

## TypeScript dialect notes

- **Types are erased.** Nothing is checked at runtime; validate external data (see app 7 with Zod).
- **`erasableSyntaxOnly`.** No `enum`, `namespace`, or constructor parameter properties, because Node strips types without compiling. Use `as const` objects with a derived union instead (app 4). Decorators are likewise unavailable when running through Node directly.
- **Structural typing.** An object satisfies an interface by shape, not by declaration. Rust traits and Python Protocols are the nearest relatives; `implements` is only a compile-time assertion.
- **`strict` plus extras.** `noUncheckedIndexedAccess` makes `arr[i]` `T | undefined`, matching `Vec::get` and `dict.get`. `exactOptionalPropertyTypes` distinguishes "missing" from "undefined".
- **ESM only.** Relative imports include the `.ts` extension; `rewriteRelativeImportExtensions` rewrites them to `.js` when building.
- **`verbatimModuleSyntax`.** Type-only imports must use `import type` so they are erased predictably.
