"""Game rules, kept free of I/O so tests can pass plain lists."""

import re
from collections.abc import Callable, Iterable

from guessing_game.models import (
    GameConfig,
    GameOutcome,
    Guess,
    Hint,
    InvalidInput,
    Lost,
    PlayerInput,
    Quit,
    QuitInput,
    Won,
)

_MASK = 0xFFFFFFFF
_DIGITS = re.compile(r"[0-9]+")  # `\d` would also match non-ASCII digits


def _imul(a: int, b: int) -> int:
    return (a * b) & _MASK


def create_rng(seed: int) -> Callable[[], float]:
    """Deterministic 32-bit PRNG (mulberry32). Same seed gives the same sequence in any language.

    Returns a closure; `nonlocal` lets it rebind `state` in the enclosing scope.
    """
    state = seed & _MASK

    def rng() -> float:
        nonlocal state
        state = (state + 0x6D2B79F5) & _MASK
        t = state
        t = _imul(t ^ (t >> 15), t | 1)
        t ^= (t + _imul(t ^ (t >> 7), t | 61)) & _MASK
        return ((t ^ (t >> 14)) & _MASK) / 4294967296

    return rng


def pick_secret(rng: Callable[[], float], config: GameConfig) -> int:
    """Pick an integer in [min, max] inclusive."""
    return config.min + int(rng() * (config.max - config.min + 1))


def judge(guess: int, secret: int) -> Hint:
    """Compare a guess with the secret."""
    if guess < secret:
        return "low"
    if guess > secret:
        return "high"
    return "correct"


def describe_hint(hint: Hint) -> str:
    """Text shown for a hint."""
    match hint:
        case "low":
            return "Too low"
        case "high":
            return "Too high"
        case "correct":
            return "Correct!"


def parse_input(line: str, config: GameConfig) -> PlayerInput:
    """`q`/`quit` ends the game; digits in range are guesses; everything else is invalid."""
    text = line.strip().lower()
    if text in ("q", "quit"):
        return QuitInput()
    if not _DIGITS.fullmatch(text):
        return InvalidInput()
    value = int(text)
    return Guess(value) if config.min <= value <= config.max else InvalidInput()


def intro(config: GameConfig) -> str:
    """Opening message."""
    return (
        f"Guess a number between {config.min} and {config.max}. "
        f"You have {config.max_attempts} attempts. Type 'q' to quit."
    )


def play_game(
    secret: int,
    config: GameConfig,
    lines: Iterable[str],
    write: Callable[[str], None],
) -> GameOutcome:
    """Drive one game. Invalid input does not use up an attempt.

    `lines` may be a list (tests) or `sys.stdin` (CLI): both are iterables of str.
    """
    attempts = 0

    for line in lines:
        match parse_input(line, config):
            case QuitInput():
                write("Bye")
                return Quit()
            case InvalidInput():
                write(f"Please enter a whole number between {config.min} and {config.max}")
                continue
            case Guess(value):
                attempts += 1
                hint = judge(value, secret)
                if hint == "correct":
                    plural = "" if attempts == 1 else "s"
                    write(f"Correct! You won in {attempts} attempt{plural}")
                    return Won(attempts)
                write(describe_hint(hint))
                if attempts >= config.max_attempts:
                    break

    # Reached by running out of attempts, or by input ending before the game did.
    if attempts >= config.max_attempts:
        write(f"Out of attempts. The number was {secret}")
        return Lost(secret)
    write("Bye")
    return Quit()


def solve(secret: int, config: GameConfig) -> int:
    """Binary-search bot: a `while` loop that halves the range each round. Returns attempts used."""
    low, high = config.min, config.max
    attempts = 0

    while low <= high:
        guess = (low + high) // 2
        attempts += 1
        hint = judge(guess, secret)
        if hint == "correct":
            break
        if hint == "low":
            low = guess + 1
        else:
            high = guess - 1
    return attempts
