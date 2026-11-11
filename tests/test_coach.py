import asyncio
import json
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach import codex_cli
from phrase_gym.coach.codex_cli import CliError, CodexCli, child_environment
from phrase_gym.coach.main import CoachApi, process_once
from phrase_gym.coach.preparation import CardPreparer
from phrase_gym.coach.speech import PiperSpeech
from tests import fake_codex
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


@pytest.fixture
def cli(tmp_path: Path) -> CodexCli:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    return CodexCli(workspace, str(fake_codex.executable(tmp_path)), timeout=10)


def run_coach_once(
    client: TestClient,
    cli: CodexCli,
    text: str,
    speech: PiperSpeech | None = None,
) -> dict[str, object]:
    package_id = client.post("/api/coach/requests", json={"text": text}).json()["request_id"]
    preparer = CardPreparer(cli, cli.workspace / ".phrase-gym-coach" / "attempts")
    assert process_once(ClientCoachApi(client), preparer, speech or PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{package_id}").json()


def attempts(cli: CodexCli) -> list[Path]:
    folder = cli.workspace / ".phrase-gym-coach" / "attempts"
    return sorted(folder.iterdir()) if folder.exists() else []


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
    client: TestClient, cli: CodexCli
) -> None:
    phrase = run_coach_once(client, cli, "strongly believe — We strongly believe it works.")
    word = run_coach_once(client, cli, "reluctant")
    assert (phrase["status"], phrase["added_count"]) == ("added", 1)
    assert (word["status"], word["added_count"]) == ("added", 1)
    texts = sorted(item["text"] for item in coach_phrases(client))
    assert texts == ["She felt reluctant today.", "We strongly believe it works."]
    assert len(stored(client, "She felt reluctant today.")["cues"]) == 3
    assert attempts(cli) == []


def test_review_findings_are_applied_before_publication(client: TestClient, cli: CodexCli) -> None:
    assert run_coach_once(client, cli, "leaky")["status"] == "added"
    assert "leaky" not in stored(client, "She felt leaky today.")["hints"]


@pytest.mark.parametrize(
    ("text", "reason"),
    [
        ("insufficient zzz", "No English word or expression was found in the input."),
        ("malformed reply", "Codex returned malformed JSON during prepare"),
        (
            "usage-limit",
            "Codex reported a usage limit; submit the text again later (during prepare)",
        ),
        (
            "subscription-limit",
            "Codex reported a usage limit; submit the text again after Nov 13th, 2026 4:58 PM",
        ),
        ("stubborn", "The prepared card failed its checks: recall_hints.0: the hint gives"),
        (
            "logged-out",
            "Codex is not logged in; run `codex login` in a terminal, then submit the text again",
        ),
        ("unauthorized quota echo-transcript", "Codex exited with code 1 (during prepare)"),
    ],
)
def test_failed_preparation_keeps_its_reason_and_publishes_nothing(
    client: TestClient,
    cli: CodexCli,
    capsys: pytest.CaptureFixture[str],
    text: str,
    reason: str,
) -> None:
    status = run_coach_once(client, cli, text)
    assert status["status"] == "error"
    assert str(status["error"]).startswith(reason)
    assert coach_phrases(client) == []
    [attempt] = attempts(cli)
    printed = capsys.readouterr().out
    assert printed.startswith(f"Coach could not prepare “{text}”: {reason}")
    assert f"(details: {attempt})" in printed
    notices = client.get("/api/practice/facts").json()["notices"]
    assert [(notice["label"], notice["kind"]) for notice in notices] == [(text, "coach")]


def test_piper_failure_after_preparation_is_reported(
    client: TestClient, cli: CodexCli, capsys: pytest.CaptureFixture[str]
) -> None:
    speech = PiperSpeech(None, fake=True, fake_failures=1)
    status = run_coach_once(client, cli, "by and large", speech)
    reason = "Synthetic preparation failure; correct the setup and retry"
    assert (status["status"], status["error"]) == ("error", reason)
    assert capsys.readouterr().out == f"Coach could not prepare “by and large”: {reason}\n"
    assert run_coach_once(client, cli, "by and large")["status"] == "added"


def test_missing_cli_and_timeout_fail_with_concrete_reasons(tmp_path: Path, cli: CodexCli) -> None:
    missing = CodexCli(cli.workspace, "definitely-not-an-installed-codex")
    with pytest.raises(CliError, match="Codex CLI was not found on PATH"):
        asyncio.run(missing.ask(tmp_path / "a", "prepare", "x", {}))
    slow = CodexCli(cli.workspace, cli.executable, timeout=0.5)
    prompt = '<learner_input>\n"slow"\n</learner_input>'
    with pytest.raises(CliError, match="Codex did not finish prepare within 0 s"):
        asyncio.run(slow.ask(tmp_path / "b", "prepare", prompt, {}))
    assert (tmp_path / "b" / "prepare-stderr.log").exists()


def test_child_receives_no_api_keys(
    tmp_path: Path, cli: CodexCli, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("OPENAI_API_KEY", "secret")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "secret")
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", "/private/data")
    assert {"OPENAI_API_KEY", "ANTHROPIC_API_KEY"}.isdisjoint(child_environment())
    answer = asyncio.run(cli.ask(tmp_path / "env", "probe", "ECHO-ENVIRONMENT", {}))
    seen = set(answer["environment"])
    assert {"HOME", "PATH"} <= seen
    assert not seen & {"OPENAI_API_KEY", "ANTHROPIC_API_KEY", "PHRASE_GYM_DATA_DIR"}


def test_codex_runs_with_reads_limited_to_the_attempt_and_env_files_denied(
    tmp_path: Path, cli: CodexCli
) -> None:
    attempt = tmp_path / "attempt"
    arguments = cli.arguments(attempt, "review")
    assert arguments[:3] == ["exec", "--skip-git-repo-check", "--ephemeral"]
    profile = arguments[arguments.index('default_permissions="phrase-gym-coach"') + 2]
    assert profile == (
        "permissions.phrase-gym-coach.filesystem={"
        f'":minimal"="read", "{attempt}"="read", '
        f'"{cli.workspace}/.env"="none", "{cli.workspace}/**/.env"="none"}}'
    )
    assert "danger" not in " ".join(arguments)
    assert arguments[-1] == f"Read your task from {attempt}/review-prompt.md and execute it."
    assert arguments[arguments.index("--cd") + 1] == str(cli.workspace)


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
