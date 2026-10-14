import asyncio
from pathlib import Path

import aiohttp
import pytest
from sqlalchemy import select

from phrase_gym.contracts.content import DialogueInput, DialogueTurn
from phrase_gym.preparation.speech import SpeechAdapter, SpeechSettings
from phrase_gym.preparation.tables import PreparationRow
from phrase_gym.preparation.worker import PreparationWorker
from tests.preparation.conftest import FakeProvider


def configured_speech(media):
    return SpeechAdapter(
        media,
        SpeechSettings.model_validate(
            {
                "voices": {
                    "en": [
                        {"engine": "edge", "voice": "en-US-AriaNeural", "locale": "en-US"},
                        {"engine": "edge", "voice": "en-US-GuyNeural", "locale": "en-US"},
                    ]
                },
            }
        ),
    )


def network_failure(kind):
    if kind == "disconnect":
        return aiohttp.ServerDisconnectedError("Synthetic disconnect")
    if kind == "handshake":
        return aiohttp.WSServerHandshakeError(None, (), status=502, message="Synthetic handshake")
    return aiohttp.ClientResponseError(None, (), status=500, message="Synthetic HTTP failure")


@pytest.mark.parametrize("failure", ["disconnect", "response", "handshake"])
def test_edge_network_failure_uses_matching_fallback(prepared_runtime, monkeypatch, failure):
    _, media, _, _ = prepared_runtime
    calls = []

    class Edge:
        def __init__(self, text, voice):
            self.voice = voice
            calls.append((text, voice))

        async def save(self, path):
            if self.voice == "en-US-AriaNeural":
                raise network_failure(failure)
            Path(path).write_bytes(Path("tests/fixtures/media/tone.mp3").read_bytes())

    monkeypatch.setattr("phrase_gym.preparation.speech.edge_tts.Communicate", Edge)
    speech = configured_speech(media)
    result = asyncio.run(speech.prepare("Right.", "en", "synthetic"))
    assert [voice for _, voice in calls] == ["en-US-AriaNeural", "en-US-GuyNeural"]
    assert media.get(result).metadata.voice == "en-US-GuyNeural"
    assert media.get(result).metadata.locale == "en-US"


def test_exhausted_network_errors_are_failed_retryable_and_keep_raw_answers(
    prepared_runtime, source, draft, monkeypatch
):
    jobs, media, _, _ = prepared_runtime
    provider = FakeProvider(draft)
    outage = {"active": True}
    source.dialogues = [
        DialogueInput(
            source_id="exchange",
            title="Synthetic complete exchange",
            turns=[
                DialogueTurn(speaker="A", text="How are you?"),
                DialogueTurn(speaker="B", text=source.phrases[0].text, target_source_id="greeting"),
            ],
        )
    ]
    calls = []

    class Edge:
        def __init__(self, text, voice):
            self.text = text
            calls.append((text, voice))

        async def save(self, path):
            if outage["active"] and self.text == "How are you?":
                raise network_failure("response")
            Path(path).write_bytes(Path("tests/fixtures/media/tone.mp3").read_bytes())

    monkeypatch.setattr("phrase_gym.preparation.speech.edge_tts.Communicate", Edge)
    job = jobs.enqueue(source, {})
    worker = PreparationWorker(jobs, provider, configured_speech(media))
    assert asyncio.run(worker.once(job.id))
    failed = jobs.get(job.id)
    assert failed.status == "failed" and failed.error == "provider_or_audio_failure"
    assert failed.result["generation"]["text"] == draft.model_dump_json()
    assert failed.result["model_review"]["text"] and len(provider.calls) == 2
    target_audio = failed.result["audio"]["target:greeting"]
    assert len(calls) == 3
    with jobs.db.read() as transaction:
        row = transaction.scalar(select(PreparationRow))
        assert row.lease_token is None and row.lease_until is None
    outage["active"] = False
    jobs.retry(job.id)
    assert asyncio.run(worker.once(job.id))
    recovered = jobs.get(job.id)
    assert recovered.status == "awaiting_review"
    assert recovered.result["generation"] == failed.result["generation"]
    assert recovered.result["model_review"] == failed.result["model_review"]
    assert recovered.result["audio"]["target:greeting"] == target_audio
    assert len(calls) == 4
    assert len(provider.calls) == 2


def test_speech_cancellation_propagates_and_worker_releases_lease(
    prepared_runtime, source, draft, monkeypatch
):
    jobs, media, _, _ = prepared_runtime
    calls = []

    class Edge:
        def __init__(self, _, voice):
            calls.append(voice)

        async def save(self, _):
            raise asyncio.CancelledError()

    monkeypatch.setattr("phrase_gym.preparation.speech.edge_tts.Communicate", Edge)
    speech = configured_speech(media)
    with pytest.raises(asyncio.CancelledError):
        asyncio.run(speech.prepare("Right.", "en", "synthetic"))
    assert calls == ["en-US-AriaNeural"]
    calls.clear()
    provider = FakeProvider(draft)
    job = jobs.enqueue(source, {})
    with pytest.raises(asyncio.CancelledError):
        asyncio.run(PreparationWorker(jobs, provider, speech).once(job.id))
    assert calls == ["en-US-AriaNeural"]
    failed = jobs.get(job.id)
    assert failed.status == "failed" and failed.error.startswith("worker_interrupted")
    assert failed.result["generation"]["text"] == draft.model_dump_json()
    with jobs.db.read() as transaction:
        assert transaction.scalar(select(PreparationRow)).lease_token is None
