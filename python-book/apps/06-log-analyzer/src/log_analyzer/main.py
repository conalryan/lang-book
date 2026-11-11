"""Entry point: wiring only."""

import argparse
from collections.abc import Sequence

from log_analyzer.io import print_error, print_line, read_file_lines
from log_analyzer.logic import analyze, format_report

USAGE = "usage: log-analyzer <file> [--filter <expr>] [--print]"


def main(argv: Sequence[str] | None = None) -> int:
    """Analyze a log file and print the summary."""
    parser = argparse.ArgumentParser(prog="log-analyzer", add_help=False)
    parser.add_argument("path", nargs="?")
    parser.add_argument("-f", "--filter", dest="filter_source")
    parser.add_argument("-p", "--print", dest="print_matches", action="store_true")
    args = parser.parse_args(argv)

    try:
        if args.path is None:
            print_error(USAGE)
            return 1
        lines = read_file_lines(args.path)
        for line in format_report(
            analyze(lines, args.filter_source), print_matches=args.print_matches
        ):
            print_line(line)
    except (ValueError, OSError) as error:  # FilterSyntaxError is a ValueError
        print_error(str(error))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
