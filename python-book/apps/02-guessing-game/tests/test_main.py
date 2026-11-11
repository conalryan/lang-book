import pytest
from guessing_game.main import main


def test_auto_mode_is_reproducible(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("GUESS_SEED", "1")
    assert main(["--auto"]) == 0
    assert capsys.readouterr().out == "Bot found 63 in 6 attempts\n"
