import re
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.agents import antigravity
from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli
from phrase_gym.coach.gemini_cli import GeminiCli
from phrase_gym.coach.main import Preparer, process_once
from phrase_gym.coach.preparation import CardPreparer
from phrase_gym.coach.recognition import Recognizer
from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.coach_contracts import CoachWork
from phrase_gym.server.languages import LearningLanguage
from tests import fake_cli
from tests.coach_client import ClientCoachApi
from tests.practice_client import Practice

NO_VOICE = "У Coach пока нет голоса для этого языка. Повторите, когда он появится."


def paid_preparer(vendor: str, tmp_path: Path) -> CardPreparer:
    fake = str(fake_cli.executable(tmp_path, vendor))
    if vendor == "claude":
        claude = ClaudeCli(fake, timeout=10, attempts=tmp_path / "attempts")
        return CardPreparer(claude, claude.attempts)
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    codex = CodexCli(workspace, fake, timeout=10)
    return CardPreparer(codex, codex.attempts)


@pytest.fixture
def recognizer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Recognizer:
    monkeypatch.setattr(antigravity, "_agy_model_switches", {})
    fake = str(fake_cli.executable(tmp_path, "gemini"))
    gemini = GeminiCli(fake, timeout=10, attempts=tmp_path / "attempts", launch_delay=0)
    return Recognizer(gemini, gemini.attempts)


def send(
    client: TestClient,
    preparer: Preparer,
    text: str,
    language: str = "de",
    speech: PiperSpeech | None = None,
) -> dict[str, object]:
    sent = client.post("/api/coach/requests", json={"text": text, "learning_language": language})
    request_id = sent.json()["request_id"]
    assert process_once(ClientCoachApi(client), preparer, speech or PiperSpeech(None, fake=True))
    return client.get(f"/api/imports/{request_id}").json()


def revealed(client: TestClient, language: str = "de") -> tuple[dict, dict]:
    practice = Practice(client)
    card = practice.card(language=language)
    return card, practice.reveal(card)["answer"]


@pytest.mark.parametrize("vendor", ["claude", "codex"])
def test_a_paid_german_card_has_a_russian_meaning_layer_and_german_wording(
    client: TestClient, tmp_path: Path, vendor: str
) -> None:
    assert send(client, paid_preparer(vendor, tmp_path), "Tisch")["status"] == "added"
    card, answer = revealed(client)
    assert re.fullmatch(r"Context \d: __________ here\. \(=значение \d\)", card["cue"]["situation"])
    assert (answer["text"], answer["explanation"]) == ("Das ist Tisch.", "Цель: Tisch.")
    assert Practice(client).daily(language="en")["card"] is None


@pytest.mark.parametrize("vendor", ["claude", "codex"])
def test_an_english_card_keeps_its_english_meaning_layer(
    client: TestClient, tmp_path: Path, vendor: str
) -> None:
    assert send(client, paid_preparer(vendor, tmp_path), "reluctant", "en")["status"] == "added"
    card, _ = revealed(client, "en")
    assert re.fullmatch(r"Context \d: __________ here\. \(=sense \d\)", card["cue"]["situation"])


def test_a_free_german_noun_stays_as_typed_and_asks_for_the_whole_word(
    client: TestClient, recognizer: Recognizer
) -> None:
    assert send(client, recognizer, "Tisch")["status"] == "added"
    card, answer = revealed(client)
    assert card["cue"]["situation"] == "T___h"
    assert card["instruction"] == "Произнесите слово или фразу целиком."
    assert answer["text"] == "Tisch"


def test_english_typed_for_german_is_rejected_with_a_russian_reason(
    client: TestClient, recognizer: Recognizer
) -> None:
    assert send(client, recognizer, "I am very tired")["status"] == "error"
    [item] = client.get("/api/coach/inbox").json()["items"]
    assert (item["learning_language"], item["outcome"]) == ("de", "rejected")
    assert item["reason"] == "Это английский текст, а не немецкий."


class CountingPreparer:
    def __init__(self, inner: Preparer):
        self.inner = inner
        self.languages: list[str] = []

    async def prepare(self, work: CoachWork):  # noqa: ANN201 - mirrors the wrapped preparer
        self.languages.append(work.learning_language)
        return await self.inner.prepare(work)


def test_a_missing_german_voice_is_a_retryable_failure_while_english_keeps_working(
    client: TestClient, recognizer: Recognizer
) -> None:
    english_only = PiperSpeech(None, fake=True, fake_voices=("en",))
    preparer = CountingPreparer(recognizer)
    status = send(client, preparer, "Tisch", speech=english_only)
    assert status["status"] == "error"
    [item] = client.get("/api/coach/inbox").json()["items"]
    assert (item["learning_language"], item["outcome"], item["reason"]) == (
        "de",
        "failed",
        NO_VOICE,
    )
    [notice] = client.get("/api/practice/facts", params={"language": "en"}).json()["notices"]
    assert notice["retry_helps"]
    assert preparer.languages == []
    assert send(client, preparer, "colour", "en", english_only)["status"] == "added"
    assert preparer.languages == ["en"]
    retried = client.post(f"/api/coach/requests/{notice['package_id']}/retry").json()
    assert retried["outcome"] == "saved"
    assert process_once(ClientCoachApi(client), preparer, PiperSpeech(None, fake=True))
    assert revealed(client)[1]["text"] == "Tisch"


@pytest.mark.parametrize(
    ("vendor", "language", "typed", "corrected", "note"),
    [
        ("claude", "de", "die tür", "die Tür", "Исправлено, было: «die tür»."),
        ("codex", "en", "definately", "definitely", "Исправлено, было: «definately»."),
    ],
)
def test_a_paid_correction_is_noted_on_the_card_and_in_the_coach_inbox(  # noqa: PLR0913
    client: TestClient,
    tmp_path: Path,
    vendor: str,
    language: str,
    typed: str,
    corrected: str,
    note: str,
) -> None:
    assert send(client, paid_preparer(vendor, tmp_path), typed, language)["status"] == "added"
    _, answer = revealed(client, language)
    assert answer["explanation"].endswith(f": {corrected}. {note}")
    [item] = client.get("/api/coach/inbox").json()["items"]
    assert (item["outcome"], item["text"], item["correction"]) == (
        "corrected",
        typed,
        {"original": typed, "corrected": corrected},
    )


class SpySpeech(PiperSpeech):
    def __init__(self) -> None:
        super().__init__(None, fake=True)
        self.spoken: list[tuple[str, str]] = []

    async def synthesize(self, text: str, language: LearningLanguage) -> bytes:
        self.spoken.append((text, language))
        return await super().synthesize(text, language)


def test_coach_speaks_each_card_in_its_own_learning_language(
    client: TestClient, tmp_path: Path, recognizer: Recognizer
) -> None:
    speech = SpySpeech()
    send(client, paid_preparer("claude", tmp_path), "Tisch", speech=speech)
    send(client, recognizer, "colour", "en", speech)
    assert speech.spoken == [("Das ist Tisch.", "de"), ("colour", "en")]
