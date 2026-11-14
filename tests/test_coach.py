import asyncio
import json
import os
import time
from dataclasses import replace
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach import codex_cli
from phrase_gym.coach.agents.ai_agent import _inject_skip_git_flag
from phrase_gym.coach.agents.command_template import build_run_args
from phrase_gym.coach.claude_cli import ClaudeCli, attempts_root, template_values
from phrase_gym.coach.cli_steps import CliError
from phrase_gym.coach.codex_cli import CodexCli
from phrase_gym.coach.main import CoachApi, process_once
from phrase_gym.coach.preparation import CardPreparer
from phrase_gym.coach.speech import PiperSpeech
from tests import fake_cli
from tests.coach_client import ClientCoachApi


@pytest.mark.parametrize(
    "url",
    ["http://127.0.0.1:8000", "http://localhost:8941/"],
)
def test_coach_accepts_only_canonical_loopback_authority(url: str) -> None:
    assert CoachApi(url).base_url == url.rstrip("/")


@pytest.mark.parametrize(
    "url",
    [
        "http://127.0.0.1:8000@attacker.example",
        "http://user@localhost:8000",
        "https://localhost:8000",
        "http://attacker.example:8000",
        "http://localhost",
        "http://localhost:0",
        "http://localhost:99999",
        "http://localhost:8000/other",
        "http://localhost:8000?next=attacker.example",
    ],
)
def test_coach_rejects_non_loopback_or_ambiguous_url(url: str) -> None:
    with pytest.raises(ValueError, match="loopback"):
        CoachApi(url)


Cli = ClaudeCli | CodexCli
LABEL = {ClaudeCli: "Claude", CodexCli: "Codex"}


@pytest.fixture(params=["claude", "codex"])
def cli(request: pytest.FixtureRequest, tmp_path: Path) -> Cli:
    fake = str(fake_cli.executable(tmp_path, request.param))
    if request.param == "claude":
        return ClaudeCli(fake, timeout=10, attempts=tmp_path / "attempts")
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    return CodexCli(workspace, fake, timeout=10)


def run_coach_once(
    client: TestClient,
    cli: Cli,
    text: str,
    speech: PiperSpeech | None = None,
) -> dict[str, object]:
    package_id = client.post("/api/coach/requests", json={"text": text}).json()["request_id"]
    preparer = CardPreparer(cli, cli.attempts)
    assert process_once(ClientCoachApi(client), preparer, speech or PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{package_id}").json()


def attempts(cli: Cli) -> list[Path]:
    return sorted(cli.attempts.iterdir()) if cli.attempts.exists() else []


def coach_phrases(client: TestClient) -> list[dict[str, object]]:
    lessons = client.get("/api/lessons").json()["items"]
    coach = [lesson["id"] for lesson in lessons if lesson["title"] == "Added with Coach"]
    if not coach:
        return []
    return client.get("/api/phrases", params={"lesson_id": coach[0]}).json()["items"]


def stored(client: TestClient, text: str) -> dict[str, object]:
    with client.app.state.database.read() as connection:
        row = connection.execute(
            "SELECT cues_json, hints_json FROM phrases WHERE text=?", (text,)
        ).fetchone()
    return {"cues": json.loads(row["cues_json"]), "hints": json.loads(row["hints_json"])}


def test_typed_phrase_and_word_are_prepared_reviewed_and_published(
    client: TestClient, cli: Cli
) -> None:
    phrase = run_coach_once(client, cli, "strongly believe — We strongly believe it works.")
    word = run_coach_once(client, cli, "reluctant")
    assert (phrase["status"], phrase["added_count"]) == ("added", 1)
    assert (word["status"], word["added_count"]) == ("added", 1)
    texts = sorted(item["text"] for item in coach_phrases(client))
    assert texts == ["She felt reluctant today.", "We strongly believe it works."]
    assert len(stored(client, "She felt reluctant today.")["cues"]) == 3
    assert attempts(cli) == []


def test_review_findings_are_applied_before_publication(client: TestClient, cli: Cli) -> None:
    assert run_coach_once(client, cli, "leaky")["status"] == "added"
    assert "leaky" not in stored(client, "She felt leaky today.")["hints"]


LOGIN = "not logged in; log in to the CLI in a terminal, then submit the text again"
REASONS = {
    "Codex": {
        "malformed": "Codex returned malformed JSON during prepare",
        "usage-limit": "Codex: rate limit hit; submit the text again later (during prepare)",
        "subscription-limit": "Codex: usage limit reached; submit the text again after "
        "Nov 13th, 2026 4:58 PM (during prepare)",
        "logged-out": f"Codex: {LOGIN} (during prepare)",
    },
    "Claude": {
        "malformed": "Claude returned no structured answer during prepare",
        "usage-limit": "Claude: rate limit hit; submit the text again later (during prepare)",
        "subscription-limit": "Claude: usage limit reached; submit the text again after "
        "6pm (Europe/Belgrade) (during prepare)",
        "logged-out": f"Claude: {LOGIN} (during prepare)",
    },
}


@pytest.mark.parametrize(
    ("text", "reason"),
    [
        ("insufficient zzz", "No English word or expression was found in the input."),
        ("malformed reply", "malformed"),
        ("usage-limit", "usage-limit"),
        ("subscription-limit", "subscription-limit"),
        ("stubborn", "The prepared card failed its checks: recall_hints.0: the hint gives"),
        ("logged-out", "logged-out"),
        (
            "unauthorized quota: usage limit reached, please run /login echo-transcript",
            "exited",
        ),
    ],
)
def test_failed_preparation_keeps_its_reason_and_publishes_nothing(
    client: TestClient,
    cli: Cli,
    capsys: pytest.CaptureFixture[str],
    text: str,
    reason: str,
) -> None:
    label = LABEL[type(cli)]
    reason = REASONS[label].get(reason, reason)
    if reason == "exited":
        reason = f"{label}: agent exit code 1 (during prepare)"
    status = run_coach_once(client, cli, text)
    assert status["status"] == "error"
    assert str(status["error"]).startswith(reason)
    assert coach_phrases(client) == []
    [attempt] = attempts(cli)
    printed = capsys.readouterr().out
    assert printed.startswith(f"Coach could not prepare “{text}”: {reason}")
    assert f"(details: {attempt})" in printed
    notices = client.get("/api/practice/facts").json()["notices"]
    retry_helps = not text.startswith("insufficient")
    assert [(notice["label"], notice["kind"], notice["retry_helps"]) for notice in notices] == [
        (text, "coach", retry_helps)
    ]


def test_piper_failure_after_preparation_is_reported(
    client: TestClient, cli: Cli, capsys: pytest.CaptureFixture[str]
) -> None:
    speech = PiperSpeech(None, fake=True, fake_failures=1)
    status = run_coach_once(client, cli, "by and large", speech)
    reason = "Synthetic preparation failure; correct the setup and retry"
    assert (status["status"], status["error"]) == ("error", reason)
    assert capsys.readouterr().out == f"Coach could not prepare “by and large”: {reason}\n"
    assert run_coach_once(client, cli, "by and large")["status"] == "added"


def test_missing_cli_and_timeout_fail_with_concrete_reasons(tmp_path: Path, cli: Cli) -> None:
    label = LABEL[type(cli)]
    missing = replace(cli, executable="definitely-not-an-installed-cli")
    with pytest.raises(CliError, match=f"{label}: Agent command not found"):
        asyncio.run(missing.ask(tmp_path / "a", "prepare", "x", {}))
    slow = replace(cli, timeout=1)
    prompt = '<learner_input>\n"slow"\n</learner_input>'
    with pytest.raises(CliError, match=f"{label}: agent timed out \\(during prepare\\)"):
        asyncio.run(slow.ask(tmp_path / "b", "prepare", prompt, {}))
    assert (tmp_path / "b" / "prepare-stderr.log").exists()


def test_child_receives_no_api_keys_or_session_state(
    tmp_path: Path, cli: Cli, monkeypatch: pytest.MonkeyPatch
) -> None:
    hidden = {
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_BASE_URL",
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDECODE",
        "CLAUDE_CODE_ENTRYPOINT",
        "PHRASE_GYM_DATA_DIR",
    }
    for name in hidden:
        monkeypatch.setenv(name, "secret")
    answer = asyncio.run(cli.ask(tmp_path / "env", "probe", "ECHO-ENVIRONMENT", {}))
    seen = set(answer["environment"])
    assert {"HOME", "PATH"} <= seen
    assert not seen & hidden


def test_claude_reads_only_the_attempt_with_home_and_env_files_denied(tmp_path: Path) -> None:
    attempt = tmp_path / "attempt"
    arguments, head = build_run_args(
        ClaudeCli().command_template,
        model="",
        prompt_file=attempt / "review-prompt.md",
        **template_values(attempt, {"type": "object"}),
    )
    assert isinstance(arguments, list) and head == "claude"
    assert arguments[:6] == [
        "claude",
        "-p",
        "--output-format",
        "json",
        "--json-schema",
        '{"type": "object"}',
    ]
    for flag, value in [
        ("--tools", "Read"),
        ("--allowed-tools", f"Read(/{attempt}/**)"),
        ("--permission-mode", "manual"),
        ("--permission-prompts", "none"),
        ("--setting-sources", "user"),
    ]:
        assert arguments[arguments.index(flag) + 1] == value
    denied = arguments[
        arguments.index("--disallowed-tools") + 1 : arguments.index("--permission-mode")
    ]
    assert denied == ["Read(~/**)", "Read(//**/.env)", "Read(//**/.env.*)"]
    assert {"--strict-mcp-config", "--no-session-persistence"} <= set(arguments)
    unwanted = {"--bare", "--model", "--dangerously-skip-permissions", "dontAsk", "auto"}
    assert not unwanted & set(arguments)
    assert arguments[-2:] == [
        "--",
        f"Read your task from {attempt}/review-prompt.md and execute it.",
    ]


def test_claude_attempts_live_outside_the_checkout_so_no_repository_instructions_load() -> None:
    checkout = Path(__file__).resolve().parents[1]
    assert not attempts_root().is_relative_to(checkout)
    assert not attempts_root().is_relative_to(Path.home())


def test_codex_runs_with_reads_limited_to_the_attempt_and_env_files_denied(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    attempt = tmp_path / "attempt"
    codex = CodexCli(workspace)
    arguments, _ = build_run_args(
        codex.command_template,
        model="",
        prompt_file=attempt / "review-prompt.md",
        **codex.template_values(attempt, "review"),
    )
    assert isinstance(arguments, list)
    arguments = _inject_skip_git_flag(arguments)
    assert arguments[:4] == ["codex", "exec", "--skip-git-repo-check", "--ephemeral"]
    profile = arguments[arguments.index('default_permissions="phrase-gym-coach"') + 2]
    assert profile == (
        "permissions.phrase-gym-coach.filesystem={"
        f'":minimal"="read", "/tmp"="read", "{attempt}"="read", '
        f'"{workspace}/.env"="none", "{workspace}/**/.env"="none"}}'
    )
    assert "danger" not in " ".join(arguments)
    assert arguments[-1] == f"Read your task from {attempt}/review-prompt.md and execute it."
    assert arguments[arguments.index("--cd") + 1] == str(workspace)


def test_workspace_is_the_override_then_the_source_checkout_then_a_private_cache(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PHRASE_GYM_COACH_WORKSPACE", str(tmp_path / "chosen"))
    assert codex_cli.coach_workspace() == tmp_path / "chosen"
    monkeypatch.delenv("PHRASE_GYM_COACH_WORKSPACE")
    assert codex_cli.coach_workspace() == Path(__file__).resolve().parents[1]
    installed = tmp_path / "lib" / "site-packages" / "phrase_gym" / "coach" / "codex_cli.py"
    installed.parent.mkdir(parents=True)
    monkeypatch.setattr(codex_cli, "__file__", str(installed))
    monkeypatch.setattr(codex_cli, "user_cache_path", lambda name: tmp_path / "cache" / name)
    assert codex_cli.coach_workspace() == tmp_path / "cache" / "phrase-gym-coach"


def test_cancelling_a_step_stops_the_cli_child_promptly(tmp_path: Path, cli: Cli) -> None:
    attempt = tmp_path / "stop"
    prompt = '<learner_input>\n"slow"\n</learner_input>'

    async def cancel_while_running() -> None:
        task = asyncio.create_task(cli.ask(attempt, "prepare", prompt, {}))
        stderr = attempt / "prepare-stderr.log"
        for _ in range(100):
            await asyncio.sleep(0.1)
            if stderr.exists() and stderr.read_text().strip():
                break
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task

    started = time.monotonic()
    asyncio.run(cancel_while_running())
    # Without the stop event, shutdown waits for the step timeout (10 s) instead.
    assert time.monotonic() - started < 5
    pid = int((attempt / "prepare-stderr.log").read_text().split()[0])
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline and process_exists(pid):
        time.sleep(0.1)
    assert not process_exists(pid)


def process_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True
