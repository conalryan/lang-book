import pytest
from guessing_game.logic import (
    create_rng,
    describe_hint,
    intro,
    judge,
    parse_input,
    pick_secret,
    play_game,
    solve,
)
from guessing_game.models import (
    DEFAULT_CONFIG,
    GameConfig,
    GameOutcome,
    Guess,
    InvalidInput,
    Lost,
    PlayerInput,
    Quit,
    QuitInput,
    Won,
)

CONFIG = GameConfig(min=1, max=100, max_attempts=3)


def play(secret: int, inputs: list[str]) -> tuple[GameOutcome, list[str]]:
    output: list[str] = []
    outcome = play_game(secret, CONFIG, inputs, output.append)
    return outcome, output


def test_rng_is_deterministic() -> None:
    a, b = create_rng(42), create_rng(42)
    assert [a(), a(), a()] == [b(), b(), b()]


def test_pick_secret_stays_in_range() -> None:
    rng = create_rng(7)
    values = [pick_secret(rng, DEFAULT_CONFIG) for _ in range(1000)]
    assert min(values) >= 1
    assert max(values) <= 100


def test_matches_cross_language_reference_value() -> None:
    assert pick_secret(create_rng(1), DEFAULT_CONFIG) == 63


def test_judge_and_describe_hint() -> None:
    hints = [judge(1, 5), judge(9, 5), judge(5, 5)]
    assert hints == ["low", "high", "correct"]
    assert [describe_hint(h) for h in hints] == ["Too low", "Too high", "Correct!"]


@pytest.mark.parametrize(
    ("line", "expected"),
    [
        ("50", Guess(50)),
        (" 7 \n", Guess(7)),
        ("Q", QuitInput()),
        ("quit", QuitInput()),
        ("0", InvalidInput()),
        ("101", InvalidInput()),
        ("-3", InvalidInput()),
        ("abc", InvalidInput()),
        ("", InvalidInput()),
        ("\uff15\uff10", InvalidInput()),  # full-width digits are not ASCII digits
    ],
)
def test_parse_input(line: str, expected: PlayerInput) -> None:
    assert parse_input(line, CONFIG) == expected


def test_win_counts_attempts() -> None:
    outcome, output = play(30, ["10", "50", "30"])
    assert outcome == Won(3)
    assert output == ["Too low", "Too high", "Correct! You won in 3 attempts"]


def test_singular_attempt() -> None:
    assert play(30, ["30"])[1] == ["Correct! You won in 1 attempt"]


def test_invalid_input_is_free() -> None:
    outcome, output = play(30, ["abc", "999", "30"])
    assert outcome == Won(1)
    assert output[:2] == ["Please enter a whole number between 1 and 100"] * 2


def test_loses_after_max_attempts_and_ignores_remaining_input() -> None:
    outcome, output = play(30, ["1", "2", "3", "30"])
    assert outcome == Lost(30)
    assert output[-1] == "Out of attempts. The number was 30"


def test_quits_on_request_and_when_input_ends() -> None:
    assert play(30, ["q"])[0] == Quit()
    assert play(30, ["5"])[0] == Quit()


def test_accepts_any_iterable() -> None:
    output: list[str] = []
    play_game(30, CONFIG, iter(["30\n"]), output.append)
    assert output == ["Correct! You won in 1 attempt"]


def test_solve_needs_at_most_seven_attempts() -> None:
    assert all(solve(secret, DEFAULT_CONFIG) <= 7 for secret in range(1, 101))


def test_intro() -> None:
    assert intro(DEFAULT_CONFIG) == (
        "Guess a number between 1 and 100. You have 10 attempts. Type 'q' to quit."
    )
