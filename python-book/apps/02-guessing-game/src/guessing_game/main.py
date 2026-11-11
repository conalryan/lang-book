"""Entry point: wiring only."""

import os
import sys
import time
from collections.abc import Sequence

from guessing_game.io import print_line, read_lines
from guessing_game.logic import create_rng, intro, pick_secret, play_game, solve
from guessing_game.models import DEFAULT_CONFIG


def main(argv: Sequence[str] | None = None) -> int:
    """Run the game. GUESS_SEED makes the secret reproducible in every language port."""
    args = sys.argv[1:] if argv is None else argv
    seed = int(os.environ.get("GUESS_SEED", time.time_ns() // 1_000_000))
    secret = pick_secret(create_rng(seed), DEFAULT_CONFIG)

    if "--auto" in args:
        attempts = solve(secret, DEFAULT_CONFIG)
        print_line(f"Bot found {secret} in {attempts} attempts")
    else:
        print_line(intro(DEFAULT_CONFIG))
        play_game(secret, DEFAULT_CONFIG, read_lines(), print_line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
