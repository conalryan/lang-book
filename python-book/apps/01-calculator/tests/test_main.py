import pytest
from calculator.main import main


def test_main_prints_result(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["3", "+", "4"]) == 0
    assert capsys.readouterr().out == "3 + 4 = 7\n"


def test_main_reports_errors(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["1", "/", "0"]) == 1
    captured = capsys.readouterr()
    assert captured.err == "error: division by zero\n"
    assert captured.out == ""
