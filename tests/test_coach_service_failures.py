import json
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.checked_preparation import CheckedPreparer
from phrase_gym.coach.main import process_once
from phrase_gym.coach.speech import PiperSpeech
from tests.coach_client import ClientCoachApi
from tests.coach_preparers import checked_preparer

LOGIN = "not logged in; log in to the CLI in a terminal, then submit the text again"
REASONS = {
    "codex": {
        "malformed": "Codex returned malformed JSON during write",
        "usage-limit": "Codex: rate limit hit; submit the text again later (during write)",
        "subscription-limit": "Codex: usage limit reached; submit the text again after "
        "Nov 13th, 2026 4:58 PM (during write)",
        "event-limit": "Codex: usage limit reached; submit the text again later (during write)",
        "logged-out": f"Codex: {LOGIN} (during write)",
        "exited": "Codex: agent exit code 1 (during write)",
    },
    "claude": {
        "malformed": "Claude returned no structured answer during write",
        "usage-limit": "Claude: rate limit hit; submit the text again later (during write)",
        "subscription-limit": "Claude: usage limit reached; submit the text again after "
        "6pm (Europe/Belgrade) (during write)",
        "logged-out": f"Claude: {LOGIN} (during write)",
        "exited": "Claude: agent exit code 1 (during write)",
    },
}
# A transcript that echoes the learner's text must not be read as the CLI's own error.
ECHOED = "unauthorized quota: usage limit reached, please run /login echo-transcript"


@pytest.fixture(params=["claude", "codex"])
def vendor(request: pytest.FixtureRequest) -> str:
    return request.param


@pytest.fixture
def preparer(vendor: str, tmp_path: Path) -> CheckedPreparer:
    return checked_preparer(vendor, tmp_path)


def run_once(
    client: TestClient, preparer: CheckedPreparer, text: str, speech: PiperSpeech | None = None
) -> dict[str, object]:
    package_id = client.post("/api/coach/requests", json={"text": text}).json()["request_id"]
    assert process_once(ClientCoachApi(client), preparer, speech or PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{package_id}").json()


TYPED = {
    "malformed": "malformed reply",
    "usage-limit": "usage-limit",
    "subscription-limit": "subscription-limit",
    "event-limit": "event-limit",
    "logged-out": "logged-out",
    "exited": ECHOED,
}


@pytest.mark.parametrize(
    ("vendor", "reason"),
    [(vendor, reason) for vendor, reasons in REASONS.items() for reason in reasons],
)
def test_a_service_failure_keeps_its_reason_and_publishes_nothing(
    client: TestClient,
    preparer: CheckedPreparer,
    capsys: pytest.CaptureFixture[str],
    vendor: str,
    reason: str,
) -> None:
    text = TYPED[reason]
    reason = REASONS[vendor][reason]
    status = run_once(client, preparer, text)
    assert status["status"] == "error"
    assert str(status["error"]).startswith(reason)
    [attempt] = preparer.attempts.iterdir()
    printed = capsys.readouterr().out
    assert printed.startswith(f"Coach could not prepare “{text}”: {reason}")
    assert f"(details: {attempt})" in printed
    notices = client.get("/api/practice/facts").json()["notices"]
    assert [(notice["label"], notice["kind"]) for notice in notices] == [(text, "coach")]
    [outcome] = client.get("/api/coach/inbox").json()["items"]
    assert outcome["outcome"] == "failed"
    [summary] = preparer.usage.iterdir()
    recorded = json.loads(summary.read_text())
    assert recorded["outcome"].startswith(f"failed: {reason}")
    assert [step["step"] for step in recorded["steps"]] == ["write"]


def test_piper_failure_after_preparation_is_reported_and_retry_publishes_once(
    client: TestClient, preparer: CheckedPreparer, capsys: pytest.CaptureFixture[str]
) -> None:
    speech = PiperSpeech(None, fake=True, fake_failures=1)
    status = run_once(client, preparer, "by and large", speech)
    reason = "Synthetic preparation failure; correct the setup and retry"
    assert (status["status"], status["error"]) == ("error", reason)
    assert capsys.readouterr().out == f"Coach could not prepare “by and large”: {reason}\n"
    assert run_once(client, preparer, "by and large")["status"] == "added"
    with client.app.state.database.read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM phrases").fetchone()[0] == 1
