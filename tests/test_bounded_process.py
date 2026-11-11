import asyncio
import os
import sys
from pathlib import Path

import pytest

from phrase_gym.coach.bounded_process import OutputLimitError, run_to_files


def run(tmp_path: Path, code: str, *, timeout: float = 10, limit: int = 1_000_000) -> int:
    return asyncio.run(
        run_to_files(
            Path(sys.executable),
            ["-c", code],
            cwd=tmp_path,
            environment={"PATH": os.environ["PATH"]},
            timeout=timeout,
            output=(tmp_path / "out.log", tmp_path / "err.log"),
            limit=limit,
        )
    )


def test_output_lands_in_files_with_the_exit_code(tmp_path: Path) -> None:
    code = "import sys; print('answer'); print('note', file=sys.stderr); sys.exit(3)"
    assert run(tmp_path, code) == 3
    assert (tmp_path / "out.log").read_text() == "answer\n"
    assert (tmp_path / "err.log").read_text() == "note\n"


def test_timeout_stops_the_child_and_keeps_its_log(tmp_path: Path) -> None:
    code = "import time; print('started', flush=True); time.sleep(30)"
    with pytest.raises(TimeoutError):
        run(tmp_path, code, timeout=1)
    assert (tmp_path / "out.log").read_text() == "started\n"


def test_oversized_output_stops_the_child(tmp_path: Path) -> None:
    code = "import sys, time\nwhile True: sys.stdout.write('x' * 65536); sys.stdout.flush()"
    with pytest.raises(OutputLimitError):
        run(tmp_path, code, limit=100_000)
    assert (tmp_path / "out.log").stat().st_size > 100_000
