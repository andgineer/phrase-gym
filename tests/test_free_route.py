from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.agents import antigravity
from phrase_gym.coach.gemini_cli import GeminiCli
from phrase_gym.coach.main import configured_preparer, process_once
from phrase_gym.coach.recognition import Recognizer
from phrase_gym.coach.speech import PiperSpeech
from tests import fake_cli
from tests.coach_client import ClientCoachApi
from tests.practice_client import Practice


@pytest.fixture
def recognizer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Recognizer:
    monkeypatch.setattr(antigravity, "_agy_model_switches", {})
    fake = str(fake_cli.executable(tmp_path, "gemini"))
    gemini = GeminiCli(fake, timeout=10, attempts=tmp_path / "attempts", launch_delay=0)
    return Recognizer(gemini, gemini.attempts)


def send(client: TestClient, recognizer: Recognizer, text: str) -> dict[str, object]:
    request = client.post(
        "/api/coach/requests", json={"text": text, "learning_language": "en"}
    ).json()
    assert request["outcome"] == "saved"
    api = ClientCoachApi(client)
    assert process_once(api, recognizer, PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{request['request_id']}").json()


def practised(client: TestClient) -> tuple[dict[str, object], dict[str, object]]:
    practice = Practice(client)
    card = practice.card()
    return card, practice.reveal(card)["answer"]


def inbox(client: TestClient) -> list[dict[str, object]]:
    return client.get("/api/coach/inbox").json()["items"]


def test_the_free_route_runs_one_low_flash_recognition_call() -> None:
    preparer = configured_preparer("gemini")
    assert isinstance(preparer, Recognizer)
    assert isinstance(preparer.cli, GeminiCli)
    assert preparer.cli.model == "--model gemini-3.8-flash-low"


def test_a_whole_sentence_becomes_a_source_gap(client: TestClient, recognizer: Recognizer) -> None:
    status = send(client, recognizer, "We strongly believe this will work")
    assert status["status"] == "added"
    card, answer = practised(client)
    assert card["cue"]["situation"] == "We __________ this will work"
    assert card["instruction"] == "Произнесите пропущенное."
    assert answer["text"] == "We strongly believe this will work"
    assert inbox(client) == []
    assert not list(recognizer.attempts.iterdir())


def test_a_target_with_its_source_sentence_gaps_that_sentence(
    client: TestClient, recognizer: Recognizer
) -> None:
    send(client, recognizer, "strongly believe — We strongly believe it works.")
    card, answer = practised(client)
    assert card["cue"]["situation"] == "We __________ it works."
    assert answer["text"] == "We strongly believe it works."


def test_an_inflected_target_gaps_its_form_in_the_sentence(
    client: TestClient, recognizer: Recognizer
) -> None:
    sentence = "She came up with a brilliant plan overnight."
    send(client, recognizer, f"come up with — {sentence}")
    card, answer = practised(client)
    assert card["cue"]["situation"] == "She __________ a brilliant plan overnight."
    assert answer["text"] == sentence
    [found] = client.get("/api/phrases", params={"query": "come up with"}).json()["items"]
    assert found["text"] == sentence


@pytest.mark.parametrize(
    "typed",
    ["definitely", "definitely — I will definitely come tomorow."],
)
def test_a_lone_target_or_faulty_surroundings_get_wording_recall(
    client: TestClient, recognizer: Recognizer, typed: str
) -> None:
    send(client, recognizer, typed)
    card, answer = practised(client)
    assert card["cue"]["situation"] == "def_____ly"
    assert "definitely" not in str(card["instruction"])
    assert (answer["text"], answer["explanation"]) == ("definitely", "")


def test_a_valid_variant_keeps_its_form(client: TestClient, recognizer: Recognizer) -> None:
    send(client, recognizer, "colour")
    card, answer = practised(client)
    assert (card["cue"]["situation"], answer["text"]) == ("c____r", "colour")
    assert inbox(client) == []


def test_an_obvious_typo_is_corrected_and_shown_in_coach(
    client: TestClient, recognizer: Recognizer
) -> None:
    send(client, recognizer, "definately")
    _, answer = practised(client)
    assert (answer["text"], answer["explanation"]) == ("definitely", "Corrected from “definately”.")
    [item] = inbox(client)
    assert (item["outcome"], item["text"], item["correction"]) == (
        "corrected",
        "definately",
        {"original": "definately", "corrected": "definitely"},
    )


@pytest.mark.parametrize(
    ("typed", "reason"),
    [
        ("zzqx", "Это не английское слово и не выражение."),
        ("Guten Morgen", "Это немецкий текст, а не английский."),
        ("fom", "«fom» может быть разными английскими словами, например from или form."),
    ],
)
def test_rejected_input_explains_itself_in_coach_and_is_not_restarted(
    client: TestClient, recognizer: Recognizer, typed: str, reason: str
) -> None:
    status = send(client, recognizer, typed)
    assert (status["status"], status["error"]) == ("error", reason)
    assert inbox(client) == [
        {
            "request_id": status["package_id"],
            "text": typed,
            "learning_language": "en",
            "outcome": "rejected",
            "reason": reason,
            "correction": None,
        }
    ]
    assert client.get("/api/practice/facts").json()["notices"] == []
    again = client.post(
        "/api/coach/requests", json={"text": typed, "learning_language": "en"}
    ).json()
    assert again["outcome"] == "rejected"
    assert not process_once(ClientCoachApi(client), recognizer, PiperSpeech(None, fake=True))


@pytest.mark.parametrize(
    ("typed", "reason"),
    [
        ("logged-out", "Gemini: Not logged into Antigravity — run: agy login (during recognize)"),
        ("subscription-limit", "Gemini: Antigravity weekly quota exhausted — resets in 4d 7h"),
        ("usage-limit", "Gemini: Antigravity API quota exhausted (rate limit)"),
        ("malformed", "Gemini returned no JSON object during recognize"),
    ],
)
def test_a_service_failure_is_a_separate_retryable_result_with_a_practice_notice(
    client: TestClient, recognizer: Recognizer, typed: str, reason: str
) -> None:
    status = send(client, recognizer, typed)
    assert status["status"] == "error" and str(status["error"]).startswith(reason)
    [item] = inbox(client)
    assert item["outcome"] == "failed"
    [notice] = client.get("/api/practice/facts").json()["notices"]
    assert (notice["label"], notice["retry_helps"]) == (typed, True)
    retried = client.post(f"/api/coach/requests/{status['package_id']}/retry").json()
    assert retried["outcome"] == "saved"
    assert client.get("/api/practice/facts").json()["notices"] == []


def test_failed_speech_publishes_nothing_and_stays_retryable(
    client: TestClient, recognizer: Recognizer
) -> None:
    client.post("/api/coach/requests", json={"text": "definitely", "learning_language": "en"})
    speech = PiperSpeech(None, fake=True, fake_failures=1)
    assert process_once(ClientCoachApi(client), recognizer, speech)
    assert client.get("/api/lessons").json()["all_lessons"]["phrase_count"] == 0
    [item] = inbox(client)
    assert item["outcome"] == "failed"


def test_a_target_too_long_for_a_card_is_a_native_language_decision(
    client: TestClient, recognizer: Recognizer
) -> None:
    typed = " ".join(["strongly"] * 30)
    status = send(client, recognizer, typed)
    assert (status["status"], status["error"]) == (
        "error",
        "Текст слишком длинный для одной карточки.",
    )
    [item] = inbox(client)
    assert item["outcome"] == "rejected"
    assert client.get("/api/practice/facts").json()["notices"] == []
