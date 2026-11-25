import asyncio
import itertools
import json
import sys
import threading
import time
from pathlib import Path

import pytest

from phrase_gym.coach.agents import antigravity
from phrase_gym.coach.agents.ai_agent import _run_agent_cli
from phrase_gym.coach.agents.antigravity import _flash_cost_rank, paced
from phrase_gym.coach.agents.antigravity_errors import agy_failure
from phrase_gym.coach.agents.subprocess import SubprocessResult
from phrase_gym.coach.cli_steps import CliError
from phrase_gym.coach.gemini_cli import GeminiCli, answer_object

# `agy models` output from 2026-11-15.
LISTING = (
    "Fetching available models...\n"
    "gemini-3.8-flash-high\tGemini 3.8 Flash (High)\n"
    "gemini-3.8-flash-medium\tGemini 3.8 Flash (Medium)\n"
    "gemini-3.8-flash-low\tGemini 3.8 Flash (Low)\n"
    "claude-sonnet-4-6\tClaude Sonnet 4.6 (Thinking)\n"
    "gpt-oss-120b-medium\tGPT-OSS 120B (Medium)\n"
)
REJECTION = (
    'invalid model selection (--model "{model}" --effort ""): model {model} is not '
    "recognized as a known model or custom model in settings\nAvailable models:\n  Gemini"
)


@pytest.fixture(autouse=True)
def fresh_run(monkeypatch: pytest.MonkeyPatch) -> None:
    # Model switches and launch pacing last for one Coach process.
    monkeypatch.setattr(antigravity, "_agy_model_switches", {})
    monkeypatch.setattr(antigravity, "_last_launch", [float("-inf")])


def fake_agy(directory: Path, listing: str, accepted: str) -> str:
    """Lists *listing*, answers only when asked for *accepted* and records every launch."""
    script = directory / "agy"
    script.write_text(
        f"#!{sys.executable}\n"
        "import json, sys, time\n"
        "from pathlib import Path\n"
        f"listing, accepted, rejection = {listing!r}, {accepted!r}, {REJECTION!r}\n"
        "if sys.argv[1:] == ['models']:\n"
        "    print(listing, end='')\n"
        "    sys.exit(0)\n"
        "model = sys.argv[sys.argv.index('--model') + 1]\n"
        "with (Path(__file__).parent / 'asked.txt').open('a') as asked:\n"
        "    asked.write(f'{model} {time.monotonic()} {\" \".join(sys.argv[1:-1])}\\n')\n"
        "if model != accepted:\n"
        "    error = rejection.format(model=model)\n"
        "    print(json.dumps({'status': 'ERROR', 'response': '', 'error': error}))\n"
        "    sys.exit(1)\n"
        "answer = json.dumps({'model': model})\n"
        "print(json.dumps({'status': 'SUCCESS', 'response': f'Here it is: {answer}',"
        " 'usage': {'total_tokens': 42}}))\n"
    )
    script.chmod(0o755)
    return str(script)


def asked(directory: Path) -> list[list[str]]:
    return [line.split() for line in (directory / "asked.txt").read_text().splitlines()]


def test_gemini_runs_news_recaps_antigravity_command_on_the_low_flash_model(
    tmp_path: Path,
) -> None:
    gemini = GeminiCli(fake_agy(tmp_path, LISTING, "gemini-3.8-flash-low"), launch_delay=0)
    answer = asyncio.run(gemini.ask(tmp_path / "a", "recognize", "x", {"type": "object"}))
    assert answer == {"model": "gemini-3.8-flash-low"}
    [[model, _, *arguments]] = asked(tmp_path)
    log = tmp_path.resolve() / "a" / "recognize-stderr.log"
    assert [model, *arguments] == [
        "gemini-3.8-flash-low",
        "--log-file",
        str(log),
        "--model",
        "gemini-3.8-flash-low",
        "--dangerously-skip-permissions",
        "--output-format",
        "json",
        "-p",
    ]
    assert (tmp_path / "a" / "recognize-stdout.log").read_text().startswith("Here it is: {")


@pytest.mark.parametrize(
    ("listing", "expected"),
    [
        (LISTING, "gemini-3.8-flash-low"),
        ("gemini-3.9-flash-high\tA\ngemini-3.8-flash-low\tB\n", "gemini-3.8-flash-low"),
        ("gemini-3.10-flash-low\tA\ngemini-3.9-flash-low\tB\n", "gemini-3.10-flash-low"),
        ("gemini-3.9-flash-lite-low\tA\ngemini-3.9-flash-low\tB\n", "gemini-3.9-flash-low"),
        ("gemini-4.0-flash\tA\ngemini-4.0-flash-high\tB\n", "gemini-4.0-flash"),
    ],
)
def test_the_cheapest_newest_full_flash_replaces_a_rejected_model(
    listing: str, expected: str
) -> None:
    ids = [line.split("\t")[0] for line in listing.splitlines() if "flash" in line]
    assert min(ids, key=_flash_cost_rank) == expected


def test_a_renamed_model_is_replaced_by_the_listed_flash_for_the_rest_of_the_run(
    tmp_path: Path,
) -> None:
    listing = "gemini-3.9-flash-low\tA\nclaude-opus-4-6-thinking\tB\n"
    gemini = GeminiCli(fake_agy(tmp_path, listing, "gemini-3.9-flash-low"), launch_delay=0)
    first = asyncio.run(gemini.ask(tmp_path / "a", "recognize", "x", {}))
    second = asyncio.run(gemini.ask(tmp_path / "b", "recognize", "x", {}))
    assert first == second == {"model": "gemini-3.9-flash-low"}
    assert [launch[0] for launch in asked(tmp_path)] == [
        "gemini-3.8-flash-low",
        "gemini-3.9-flash-low",
        "gemini-3.9-flash-low",
    ]


def test_a_rejected_model_with_no_other_flash_listed_fails_with_the_rejection(
    tmp_path: Path,
) -> None:
    gemini = GeminiCli(fake_agy(tmp_path, "claude-sonnet-4-6\tClaude\n", "none"), launch_delay=0)
    with pytest.raises(CliError, match="Gemini: Invalid model/effort selection"):
        asyncio.run(gemini.ask(tmp_path / "a", "recognize", "x", {}))
    assert "invalid model selection" in (tmp_path / "a" / "recognize-stderr.log").read_text()


def test_launches_run_one_at_a_time_and_start_spaced_apart() -> None:
    starts: list[float] = []
    active: list[int] = []

    def launch() -> None:
        starts.append(time.monotonic())
        active.append(1)
        assert len(active) == 1
        time.sleep(0.05)
        active.pop()

    stop = threading.Event()
    threads = [threading.Thread(target=paced, args=(launch, 0.3, stop)) for _ in range(3)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    assert len(starts) == 3
    assert all(later - earlier >= 0.3 for earlier, later in itertools.pairwise(starts))


def result_with(tmp_path: Path, exit_code: int, stderr: str, stdout: str = "") -> SubprocessResult:
    (tmp_path / "out.log").write_text(stdout)
    (tmp_path / "err.log").write_text(stderr)
    return SubprocessResult(exit_code, False, tmp_path / "out.log", tmp_path / "err.log")


@pytest.mark.parametrize(
    ("stderr", "reason"),
    [
        ("You are not logged into Antigravity.", "Gemini: Not logged into Antigravity"),
        ("Individual quota reached. Resets in 103h45m22s", "Gemini: Antigravity weekly quota"),
        (
            "RetryableQuotaError: You have exhausted your capacity on this model.",
            "Gemini: Antigravity API quota exhausted (rate limit)",
        ),
        ("fatal: something else", "Gemini: agent exit code 1"),
    ],
)
def test_antigravity_failures_are_classified_like_news_recap(
    tmp_path: Path, stderr: str, reason: str
) -> None:
    failure = agy_failure(result_with(tmp_path, 1, stderr), "Gemini")
    assert failure is not None and failure.startswith(reason)


def test_the_startup_login_line_is_ignored_once_agy_authenticates(tmp_path: Path) -> None:
    log = "You are not logged into Antigravity\nsilent auth succeeded\nfatal: oops"
    assert agy_failure(result_with(tmp_path, 1, log), "Gemini") == "Gemini: agent exit code 1"
    assert agy_failure(result_with(tmp_path, 0, log, "{}"), "Gemini") is None


@pytest.mark.parametrize(
    "response",
    ['{"a": 1}', '```json\n{"a": 1}\n```', 'Answer: {"a": 1} Done.'],
)
def test_the_answer_object_is_read_from_the_response_text(response: str) -> None:
    assert answer_object(response, "recognize") == {"a": 1}


def test_a_response_without_an_object_is_malformed() -> None:
    with pytest.raises(CliError, match="Gemini returned no JSON object during recognize"):
        answer_object("I could not decide.", "recognize")


def test_the_runner_redirects_agys_log_into_the_stderr_file(tmp_path: Path) -> None:
    script = tmp_path / "agy"
    script.write_text(f"#!{sys.executable}\nimport json, sys\nprint(json.dumps(sys.argv[1:]))\n")
    script.chmod(0o755)
    (tmp_path / "p.md").write_text("x")
    started = time.monotonic()
    result = _run_agent_cli(
        prompt_file=tmp_path / "p.md",
        stdout_path=tmp_path / "out.log",
        stderr_path=tmp_path / "err.log",
        cwd=tmp_path,
        timeout_seconds=10,
        command_template=f"{script} {{model}} -p x",
        values={"model": "--model m"},
    )
    assert time.monotonic() - started < 10
    arguments = json.loads(result.stdout_path.read_text())
    assert arguments[:2] == ["--log-file", (tmp_path / "err.log").as_posix()]
