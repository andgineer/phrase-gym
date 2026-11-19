import asyncio
import json
import sys
from pathlib import Path

import pytest

from phrase_gym.coach.agents.command_template import build_run_args
from phrase_gym.coach.cli_steps import CliError
from phrase_gym.coach.gemini_cli import GeminiCli, smartest_flash_model

# `agy models` output from 2026-11-15.
LISTING = (
    "Fetching available models...\n"
    "gemini-3.8-flash-high\tGemini 3.8 Flash (High)\n"
    "gemini-3.8-flash-medium\tGemini 3.8 Flash (Medium)\n"
    "gemini-3.8-flash-low\tGemini 3.8 Flash (Low)\n"
    "claude-sonnet-4-6\tClaude Sonnet 4.6 (Thinking)\n"
    "claude-opus-4-6-thinking\tClaude Opus 4.6 (Thinking)\n"
    "gpt-oss-120b-medium\tGPT-OSS 120B (Medium)\n"
)
REJECTION = (
    'invalid model selection (--model "{model}" --effort ""): model {model} is not '
    "recognized as a known model or custom model in settings\nAvailable models:\n  Gemini"
)


def fake_agy(directory: Path, listing: str, accepted: str) -> str:
    """Lists *listing*, answers only when asked for *accepted* and records every model asked."""
    script = directory / "agy"
    script.write_text(
        f"#!{sys.executable}\n"
        "import json, sys\n"
        "from pathlib import Path\n"
        f"listing, accepted, rejection = {listing!r}, {accepted!r}, {REJECTION!r}\n"
        "if sys.argv[1:] == ['models']:\n"
        "    print(listing, end='')\n"
        "    sys.exit(0)\n"
        "model = sys.argv[sys.argv.index('--model') + 1]\n"
        "with (Path(__file__).parent / 'asked.txt').open('a') as asked:\n"
        "    asked.write(model + '\\n')\n"
        "if model != accepted:\n"
        "    error = rejection.format(model=model)\n"
        "    print(json.dumps({'status': 'ERROR', 'response': '', 'error': error}))\n"
        "    sys.exit(1)\n"
        "print(json.dumps({'status': 'SUCCESS', 'structured_output': {'model': model}}))\n"
    )
    script.chmod(0o755)
    return str(script)


def test_gemini_runs_the_free_flash_model_without_skipping_permission_checks(
    tmp_path: Path,
) -> None:
    attempt = tmp_path / "attempt"
    gemini = GeminiCli()
    arguments, head = build_run_args(
        gemini.command_template,
        prompt_file=attempt / "review-prompt.md",
        **gemini.template_values(attempt, "review"),
    )
    assert isinstance(arguments, list) and head == "agy"
    assert arguments == [
        "agy",
        "--model",
        "gemini-3.8-flash-high",
        "--log-file",
        f"{attempt}/review-stderr.log",
        "--output-format",
        "json",
        "--json-schema",
        f"{attempt}/review-schema.json",
        "-p",
        f"Read your task from {attempt}/review-prompt.md and execute it.",
    ]


@pytest.mark.parametrize(
    ("listing", "expected"),
    [
        (LISTING, "gemini-3.8-flash-high"),
        ("gemini-3.9-flash-low\tA\ngemini-3.8-flash-high\tB\n", "gemini-3.9-flash-low"),
        ("gemini-3.10-flash-low\tA\ngemini-3.9-flash-low\tB\n", "gemini-3.10-flash-low"),
        ("gemini-3.9-flash-lite-high\tA\ngemini-3.9-flash-low\tB\n", "gemini-3.9-flash-low"),
        ("gemini-4.0-flash\tA\ngemini-4.0-flash-low\tB\n", "gemini-4.0-flash"),
        (
            "gemini-9-flash --unexpected-option\tA\ngemini-3.8-flash-high\tB\n",
            "gemini-3.8-flash-high",
        ),
        ("claude-sonnet-4-6\tClaude\n", None),
    ],
)
def test_the_newest_highest_effort_flash_is_the_smartest_free_model(
    listing: str, expected: str | None
) -> None:
    assert smartest_flash_model(listing) == expected


def test_a_renamed_model_is_replaced_by_the_listed_flash_for_the_rest_of_the_run(
    tmp_path: Path,
) -> None:
    listing = "gemini-3.9-flash-high\tA\nclaude-opus-4-6-thinking\tB\n"
    gemini = GeminiCli(fake_agy(tmp_path, listing, "gemini-3.9-flash-high"), timeout=10)
    first = asyncio.run(gemini.ask(tmp_path / "a", "prepare", "x", {}))
    second = asyncio.run(gemini.ask(tmp_path / "a", "review", "x", {}))
    assert first == second == {"model": "gemini-3.9-flash-high"}
    assert (tmp_path / "asked.txt").read_text().split() == [
        "gemini-3.8-flash-high",
        "gemini-3.9-flash-high",
        "gemini-3.9-flash-high",
    ]


def test_a_rejected_model_with_no_other_flash_listed_fails_with_the_rejection(
    tmp_path: Path,
) -> None:
    gemini = GeminiCli(fake_agy(tmp_path, "claude-sonnet-4-6\tClaude\n", "none"), timeout=10)
    with pytest.raises(CliError, match="Gemini: agent exit code 1: ERROR: invalid model selection"):
        asyncio.run(gemini.ask(tmp_path / "a", "prepare", "x", {}))
    assert json.loads((tmp_path / "a" / "prepare-stdout.log").read_text())["status"] == "ERROR"
