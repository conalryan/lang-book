.DEFAULT_GOAL := check

.PHONY: build fmt lint test coverage check

build:  ## compile everything (incremental)
	cargo build --workspace --all-targets

fmt:  ## format
	cargo fmt --all

lint:  ## rustfmt check + clippy with warnings as errors
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings

test:  ## nextest (parallel, per-test processes) + doctests, which nextest skips
	cargo nextest run --workspace
	cargo test --workspace --doc

coverage:  ## LLVM source-based coverage; entry points and I/O wiring excluded
	cargo llvm-cov nextest --workspace --ignore-filename-regex '(main|io)\.rs$$'

check: lint test  ## everything CI runs
