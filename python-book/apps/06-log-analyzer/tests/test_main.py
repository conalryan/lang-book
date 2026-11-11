from pathlib import Path

import pytest
from log_analyzer.main import main

SAMPLE = Path(__file__).parent.parent / "sample.log"


def test_main_prints_a_filtered_summary(capsys: pytest.CaptureFixture[str]) -> None:
    assert main([str(SAMPLE), "--filter", "level >= WARN"]) == 0
    assert capsys.readouterr().out.splitlines()[:2] == ["matched: 3 of 7", "levels: ERROR=2 WARN=1"]


def test_main_reports_syntax_errors(capsys: pytest.CaptureFixture[str]) -> None:
    assert main([str(SAMPLE), "-f", "level >"]) == 1
    assert (
        capsys.readouterr().err == "error: expected a value but found end of input at position 7\n"
    )


def test_main_reports_missing_files_and_usage(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["/no/such/file.log"]) == 1
    assert main([]) == 1
    assert "usage: log-analyzer" in capsys.readouterr().err
