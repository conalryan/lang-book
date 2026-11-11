"""Entry point: wiring only."""

from contacts.io import print_line, read_lines
from contacts.logic import run_session


def main() -> int:
    """Read commands from stdin until `quit` or EOF."""
    run_session(read_lines(), print_line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
