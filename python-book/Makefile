.DEFAULT_GOAL := check
APPS := $(wildcard apps/*)

.PHONY: install fmt lint typecheck test check

install:  ## install the pinned Python and every workspace member
	uv sync

fmt:  ## format and autofix
	uv run ruff format .
	uv run ruff check --fix .

lint:  ## lint and check formatting
	uv run ruff check .
	uv run ruff format --check .

typecheck:  ## mypy --strict, one app at a time (each has its own tests/ module names)
	@for app in $(APPS); do uv run mypy $$app/src $$app/tests || exit 1; done

test:  ## pytest with coverage across all apps
	uv run pytest --cov --cov-report=term-missing:skip-covered

check: lint typecheck test  ## everything CI runs
