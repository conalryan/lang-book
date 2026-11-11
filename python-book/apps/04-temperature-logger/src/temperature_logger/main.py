"""Entry point: wiring only."""

from temperature_logger.io import print_line, read_lines
from temperature_logger.logic import report


def main() -> int:
    """Print a report for the readings on stdin."""
    for line in report(read_lines()):
        print_line(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
