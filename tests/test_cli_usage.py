import asyncio
import json
from pathlib import Path

import pytest

from phrase_gym.coach.cli_steps import CliError
from phrase_gym.coach.usage import claude_usage, codex_usage, gemini_usage, model_and_effort
from tests.coach_preparers import subscription_writer


@pytest.mark.parametrize(
    ("flags", "model", "effort"),
    [
        ("--model claude-haiku-5-5 --effort low", "claude-haiku-5-5", "low"),
        ("--model gpt-5.6-terra -c model_reasoning_effort=low", "gpt-5.6-terra", "low"),
        ("--model gemini-3.8-flash-low", "gemini-3.8-flash-low", "low"),
    ],
)
def test_every_step_records_its_requested_model_and_effort(
    flags: str, model: str, effort: str
) -> None:
    assert model_and_effort(flags) == {"model": model, "effort": effort}


def test_codex_usage_sums_its_turn_events(tmp_path: Path) -> None:
    usage = {
        "input_tokens": 100,
        "cached_input_tokens": 40,
        "cache_write_input_tokens": 0,
        "output_tokens": 7,
        "reasoning_output_tokens": 3,
    }
    events = [
        {"type": "item.completed", "item": {"type": "agent_message", "text": "turn.completed"}},
        {"type": "turn.completed", "usage": usage},
        {"type": "turn.completed", "usage": usage},
    ]
    stdout = tmp_path / "stdout.log"
    stdout.write_text("\n".join(json.dumps(event, separators=(",", ":")) for event in events))
    assert codex_usage(stdout, tmp_path / "missing.log") == {
        "turns": 2,
        "input_tokens": 120,
        "cache_read_tokens": 80,
        "output_tokens": 14,
        "reasoning_tokens": 6,
    }


def test_codex_plain_transcript_reports_its_total_as_news_recap_reads_it(tmp_path: Path) -> None:
    stderr = tmp_path / "stderr.log"
    stderr.write_text("codex\n{}\ntokens used\n23,138\n")
    assert codex_usage(tmp_path / "stdout.log", stderr) == {"total_tokens": 23138}


def test_claude_usage_names_cache_and_thinking_tokens() -> None:
    envelope = {
        "num_turns": 3,
        "total_cost_usd": 0.0065,
        "usage": {
            "input_tokens": 4,
            "cache_creation_input_tokens": 29824,
            "cache_read_input_tokens": 29278,
            "output_tokens": 520,
            "output_tokens_details": {"thinking_tokens": 202},
        },
        "modelUsage": {"claude-haiku-5-5": {}},
    }
    assert claude_usage(envelope) == {
        "input_tokens": 4,
        "cache_read_tokens": 29278,
        "cache_write_tokens": 29824,
        "output_tokens": 520,
        "reasoning_tokens": 202,
        "turns": 3,
        "reported_models": ["claude-haiku-5-5"],
        "list_price_usd": 0.0065,
    }


@pytest.mark.parametrize("vendor", ["claude", "codex"])
def test_a_failed_step_still_records_its_model_and_time(vendor: str, tmp_path: Path) -> None:
    cli = subscription_writer(vendor, tmp_path)
    attempt = tmp_path / "attempt"
    prompt = '<learner_input>\n{"text": "logged-out"}\n</learner_input>'
    with pytest.raises(CliError):
        asyncio.run(cli.ask(attempt, "write", prompt, {}))
    record = json.loads((attempt / "write-usage.json").read_text())
    assert (record["step"], record["effort"]) == ("write", "low")
    assert record["model"] in {"claude-haiku-5-5", "gpt-5.6-luna"}
    assert record["elapsed_seconds"] > 0


ADDITIVE = ("input_tokens", "cache_read_tokens", "cache_write_tokens", "output_tokens")


def _consumed(recorded: dict[str, int]) -> int:
    return sum(recorded.get(name, 0) for name in ADDITIVE)


def test_every_cli_counts_each_token_once(tmp_path: Path) -> None:
    claude = claude_usage(
        {
            "usage": {
                "input_tokens": 4,
                "cache_read_input_tokens": 300,
                "cache_creation_input_tokens": 50,
                "output_tokens": 20,
            },
        },
    )
    assert _consumed(claude) == 4 + 300 + 50 + 20
    stdout = tmp_path / "stdout.log"
    turn = {"input_tokens": 1000, "cached_input_tokens": 600, "output_tokens": 20}
    stdout.write_text(json.dumps({"type": "turn.completed", "usage": turn}, separators=(",", ":")))
    assert _consumed(codex_usage(stdout, tmp_path / "stderr.log")) == 1000 + 20
    # The Antigravity envelope from a benchmark run: its total is input plus output.
    gemini = gemini_usage(
        {
            "input_tokens": 15185,
            "cache_read_tokens": 10665,
            "output_tokens": 118,
            "total_tokens": 15303,
        }
    )
    assert _consumed(gemini) == gemini["total_tokens"] == 15303
