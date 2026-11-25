import json
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.checked_preparation import CheckedPreparer
from phrase_gym.coach.main import configured_preparer, process_once
from phrase_gym.coach.speech import PiperSpeech
from tests.coach_client import ClientCoachApi
from tests.coach_preparers import checked_preparer
from tests.practice_client import Practice

MODELS = {
    "claude": [("write", "claude-haiku-5-5", "low"), ("check", "claude-sonnet-5", "low")],
    "codex": [("write", "gpt-5.6-luna", "low"), ("check", "gpt-5.6-terra", "low")],
}
UNRESOLVED = "Coach could not confirm one form for this input."


class SpokenSpeech(PiperSpeech):
    def __init__(self) -> None:
        super().__init__(None, fake=True)
        self.spoken: list[str] = []

    async def synthesize(self, text: str) -> bytes:
        self.spoken.append(text)
        return await super().synthesize(text)


@pytest.fixture(params=["claude", "codex"])
def vendor(request: pytest.FixtureRequest) -> str:
    return request.param


@pytest.fixture
def preparer(vendor: str, tmp_path: Path) -> CheckedPreparer:
    return checked_preparer(vendor, tmp_path)


def send(
    client: TestClient,
    preparer: CheckedPreparer,
    text: str,
    speech: PiperSpeech | None = None,
) -> dict[str, object]:
    request = client.post("/api/coach/requests", json={"text": text}).json()
    api = ClientCoachApi(client)
    assert process_once(api, preparer, speech or PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{request['request_id']}").json()


def practised(client: TestClient) -> tuple[dict[str, object], dict[str, object]]:
    practice = Practice(client)
    card = practice.card()
    return card, practice.reveal(card)["answer"]


def stored(client: TestClient) -> dict[str, object]:
    with client.app.state.database.read() as connection:
        row = connection.execute("SELECT text, cues_json, hints_json FROM phrases").fetchone()
    return {
        "text": row["text"],
        "cues": [cue["situation"] for cue in json.loads(row["cues_json"])],
        "hints": json.loads(row["hints_json"]),
    }


def inbox(client: TestClient) -> list[dict[str, object]]:
    return client.get("/api/coach/inbox").json()["items"]


def notices(client: TestClient) -> list[dict[str, object]]:
    return client.get("/api/practice/facts").json()["notices"]


def phrase_count(client: TestClient) -> int:
    with client.app.state.database.read() as connection:
        return connection.execute("SELECT COUNT(*) FROM phrases").fetchone()[0]


def test_each_subscription_writes_small_and_checks_with_a_stronger_model(vendor: str) -> None:
    preparer = configured_preparer(vendor)
    assert isinstance(preparer, CheckedPreparer)
    expected = {
        "claude": ("--model claude-haiku-5-5 --effort low", "--model claude-sonnet-5 --effort low"),
        "codex": (
            "--model gpt-5.6-luna -c model_reasoning_effort=low",
            "--model gpt-5.6-terra -c model_reasoning_effort=low",
        ),
    }
    assert (preparer.writer.model, preparer.checker.model) == expected[vendor]


def test_a_verified_example_reaches_practice_with_its_gap_gloss_and_reference(
    client: TestClient, preparer: CheckedPreparer, vendor: str
) -> None:
    speech = SpokenSpeech()
    status = send(client, preparer, "strongly believe — We strongly believe it works.", speech)
    assert (status["status"], status["added_count"]) == ("added", 1)
    assert stored(client) == {
        "text": "We strongly believe it works.",
        "cues": [
            "We __________ it works. (=sense 1)",
            "My friend said __________ at the meeting. (=sense 2)",
        ],
        "hints": ["stro... / morning", "str... / garden", "stro... / station"],
    }
    card, answer = practised(client)
    assert card["cue"]["situation"] in stored(client)["cues"]
    assert "strongly" not in json.dumps(card)
    assert card["instruction"] is None and card["hint_available"]
    assert answer["text"] == "We strongly believe it works."
    assert answer["explanation"] == "Target: strongly believe. A useful way to say it."
    assert speech.spoken == ["We strongly believe it works."]
    assert inbox(client) == []
    attempts = preparer.attempts
    assert [path.name for path in attempts.iterdir()] == ["usage"]
    [summary] = (attempts / "usage").iterdir()
    recorded = json.loads(summary.read_text())
    assert recorded["outcome"] == "enriched"
    steps = recorded["steps"]
    assert [(step["step"], step["model"], step["effort"]) for step in steps] == MODELS[vendor]
    assert all(step["usage"]["output_tokens"] == 90 for step in steps)
    assert all(step["usage"]["reasoning_tokens"] == 25 for step in steps)
    assert all(step["elapsed_seconds"] >= 0 for step in steps)


def test_without_a_usable_source_the_reference_is_a_checked_new_example(
    client: TestClient, preparer: CheckedPreparer
) -> None:
    speech = SpokenSpeech()
    assert send(client, preparer, "reluctant", speech)["status"] == "added"
    assert stored(client)["text"] == "My friend said reluctant at the meeting."
    assert speech.spoken == ["My friend said reluctant at the meeting."]


def test_an_agreed_correction_is_practised_and_shown_in_coach(
    client: TestClient, preparer: CheckedPreparer
) -> None:
    assert send(client, preparer, "definately")["status"] == "added"
    _, answer = practised(client)
    assert answer["text"] == "My friend said definitely at the meeting."
    assert answer["explanation"].endswith("Corrected from “definately”.")
    [item] = inbox(client)
    assert (item["outcome"], item["correction"]) == (
        "corrected",
        {"original": "definately", "corrected": "definitely"},
    )


def test_a_wrong_example_is_dropped_and_the_rest_of_the_card_stays(
    client: TestClient, preparer: CheckedPreparer
) -> None:
    send(client, preparer, "reluctant — She was reluctant to go, unsafe-example.")
    assert stored(client)["cues"] == [
        "She was __________ to go, unsafe-example. (=sense 1)",
        "We heard __________ again on the radio today. (=sense 3)",
    ]


def test_a_wrong_hint_is_dropped(client: TestClient, preparer: CheckedPreparer) -> None:
    send(client, preparer, "strongly believe — We strongly believe it works, unsafe-hint.")
    assert stored(client)["hints"] == ["str... / garden", "stro... / station"]


@pytest.mark.parametrize(
    "typed",
    [
        "strongly believe — We strongly believe it works, unsafe-meaning.",
        "strongly believe — We strongly believe it works, unsafe-new.",
    ],
    ids=["meaning", "every new example"],
)
def test_when_no_checked_addition_remains_basic_practice_keeps_the_target(
    client: TestClient, preparer: CheckedPreparer, typed: str
) -> None:
    assert send(client, preparer, typed)["status"] == "added"
    card, answer = practised(client)
    assert card["cue"]["situation"].startswith("We __________ it works")
    assert card["instruction"] == "Say the missing wording."
    assert not card["hint_available"]
    assert (answer["text"], answer["explanation"]) == (typed.split(" — ")[1], "")


def test_an_accurate_short_translation_is_published_unpadded(
    client: TestClient, preparer: CheckedPreparer
) -> None:
    send(client, preparer, "terse poison")
    assert practised(client)[1]["explanation"] == "яд"


@pytest.mark.parametrize(
    "typed",
    [
        "strongly believe — We strongly believe it works, check-target.",
        "strongly believe — We strongly believe it works, write-rejects.",
        "strongly believe — We strongly believe it works, check-rejects.",
    ],
    ids=["different targets", "disputed rejection", "disputed acceptance"],
)
def test_disagreement_ends_as_an_unresolved_coach_result(
    client: TestClient, preparer: CheckedPreparer, typed: str
) -> None:
    status = send(client, preparer, typed)
    assert (status["status"], status["error"]) == ("error", UNRESOLVED)
    [item] = inbox(client)
    assert (item["text"], item["outcome"], item["reason"]) == (typed, "rejected", UNRESOLVED)
    assert phrase_count(client) == 0
    assert notices(client) == []
    again = client.post("/api/coach/requests", json={"text": typed}).json()
    assert again["outcome"] == "rejected"


def test_an_agreed_rejection_keeps_its_reason(
    client: TestClient, preparer: CheckedPreparer
) -> None:
    status = send(client, preparer, "Guten Morgen")
    assert status["error"] == "This is German, not English."
    assert inbox(client)[0]["outcome"] == "rejected"


@pytest.mark.parametrize(
    ("typed", "reason"),
    [
        ("by and large sluggish", "agent timed out (during check)"),
        ("by and large checker-limit", "usage limit reached"),
        ("by and large garbled", "during check"),
        ("by and large malformed", "during write"),
    ],
    ids=["checker timeout", "checker unavailable", "malformed check", "malformed draft"],
)
def test_an_unfinished_check_is_a_service_failure_and_nothing_is_published(
    client: TestClient, vendor: str, tmp_path: Path, typed: str, reason: str
) -> None:
    preparer = checked_preparer(vendor, tmp_path, timeout=2)
    status = send(client, preparer, typed)
    assert status["status"] == "error" and reason in str(status["error"])
    assert phrase_count(client) == 0
    [item] = inbox(client)
    assert item["outcome"] == "failed"
    assert [notice["label"] for notice in notices(client)] == [typed]
    [attempt] = [path for path in preparer.attempts.iterdir() if path.name != "usage"]
    assert (attempt / "write-prompt.md").exists()
    retried = client.post(f"/api/coach/requests/{item['request_id']}/retry").json()
    assert retried["outcome"] == "saved"


def test_a_malformed_check_judging_too_few_items_never_publishes(
    client: TestClient, preparer: CheckedPreparer, monkeypatch: pytest.MonkeyPatch
) -> None:
    ask = preparer.checker.ask

    async def short_check(*arguments: object) -> dict[str, object]:
        answer = await ask(*arguments)  # type: ignore[arg-type]
        return {**answer, "hints_ok": answer["hints_ok"][:1]}

    monkeypatch.setattr(preparer, "checker", type("Short", (), {"ask": staticmethod(short_check)}))
    status = send(client, preparer, "by and large")
    assert status["error"] == "The check answer is malformed: hints_ok judges 1 of 3 items"
    assert phrase_count(client) == 0
