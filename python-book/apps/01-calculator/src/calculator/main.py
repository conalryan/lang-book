"""Entry point: wiring only."""

import sys
from collections.abc import Sequence

from calculator.io import print_error, print_line
from calculator.logic import execute, parse_command
from calculator.models import CalculatorError


def main(argv: Sequence[str] | None = None) -> int:
    """Run the CLI and return the process exit code."""
    args = list(sys.argv[1:] if argv is None else argv)
    try:
        print_line(execute(parse_command(args)))
    except CalculatorError as error:  # catch the specific type, never a bare `except:`
        print_error(str(error))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
