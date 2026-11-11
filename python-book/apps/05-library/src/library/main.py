"""Entry point: wiring only."""

from library.demo import run_demo
from library.io import print_line


def main() -> int:
    """Run the scripted demo."""
    run_demo(print_line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
