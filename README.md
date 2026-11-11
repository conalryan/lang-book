# lang-book

A common reference implementation of simple applications/libaries in various programming languages.

## Languages

- [Typescript](./typescript-book/README.md)
- [Python](./python-book/README.md)
- [Rust](./_rust-book/README.md)

## Objective
A multi-language, multi-project/package repository that contains concrete examples of the language's features including:
- data types
- variables
- general syntax - brackets, semicolons, common keywords, etc.
- assignment
- control flow
- functions
- classes
- tuples
- enums
- decorators
- etc.

## Audience
Senior full stack engineers that have decades of experiences using different languages and programming paradigms

## Requirements
1. The tech stack and development environment should use the latest, optimal, production ready tooling including:
    1. Language version management
    2. Package manager
    3. Build caching
    4. Test execution, caching and coverage
    5. Formatting, linting and typechecking
2. Workspace organization should follow common, optimal, production grade repositories
3. Projects/packages should follow common, optimal, production grade application and library separation of converns, design patterns and code/folder organization
4. The projects/packages should include runnable pseudo real world examples to demonstrate the topics in a quick, easy to step through, consice and will documented mini-application/library
5. The examples can combine similar topics to demonstrate multiple topics that are at a similar complexity or standard education sequencing combination 
6. A README.md should be included with each project/package that quickly outlines the most common patterns, features, and practices
7. The audience is a senior engineer with over a decade of engineering practice, the goal is to refresh and review the language and leverage experience in other languages to advance quickly

Great way to learn — building the same small programs in TypeScript, Python, and Rust is one of the best mental exercises because differences surface naturally (ownership, type systems, error handling). Here's a recommended order, driven by a simple principle: **each step should be a program you couldn't write before, using exactly one new concept cluster.**

# Recommended progression (7 apps)

**1. CLI calculator / unit converter** — the "hello world plus" app
Covers: entry point, printing/output, basic literals, primitive data types, variables, assignment, arithmetic, simple input parsing.
*Why first:* forces you to learn toolchain + build/run loop before language features matter.

**2. Number-guessing game** — control flow
Covers: conditionals (`if/match/switch`), loops (`while/for`), comparison operators, boolean logic, early exits, `break`/`continue`.

**3. Contact book / todo list (in-memory)** — collections + functions
Covers: arrays/lists, dictionaries/maps, iteration over collections, defining and calling functions, parameters, return values. Introduce tuples here too (returning pairs, destructuring).

**4. Temperature logger / statistics tool** — structs, classes, enums
Covers: grouping data (struct/class), methods, enums (including Rust's tagged enums vs TS/Python enum difference — very instructive), optionality/nullability (`Option<T>`, `Optional<T>`, `T | None`), error handling (exceptions vs `Result`).

**5. Library checkout / inventory system** — composition & polymorphism
Covers: multiple interacting types, traits/interfaces/protocols, inheritance vs composition, generics (a generic repository/container), impl blocks vs class methods.

**6. Log-file analyzer / mini interpreter** — advanced patterns
Covers: string manipulation, iterators/closures/lambdas, pattern matching on complex data, modules/namespaces, maybe a recursive-descent parser (great Rust ownership workout).

**7. Final capstone: a small REST-ish CLI/server app** — putting it together
Covers: project organization, dependencies/package management, visibility (`pub`/`export`), async basics if desired, testing, doc comments.

## Template conventions for comparability

To make cross-language comparison meaningful, fix these constraints for every app:

- Same file layout: one file per concept (`main`, `models`, `logic`, `io`), even if it's overkill — so you can diff structurally.
- Doc comments on every public function/type explaining *what it does* and *any language-specific gotcha* (e.g., "Rust: this consumes ownership").
- Same input/output format (same commands, same messages) so behavior diffs are zero and only code diffs remain.
- Unit tests demonstrating each public API — tests are the fastest way to relearn type signatures.

## Practical tips

- Aim for 30–90 minutes per app per language; the moment one takes 3 hours, that's the concept to drill.
- Do it in your strongest language first (build the reference), then port — ports are faster and reveal differences.
- Learn idioms deliberately: after porting, rewrite once "idiomatically" in each language. The non-idiomatic vs idiomatic diff is where deep learning happens (e.g., `Result` + `?` vs try/catch vs exceptions).
- Don't chase parity on language-specific wins (Rust lifetimes, TS unions, Python decorators) — note them in a "dialect notes" section per chapter instead.
