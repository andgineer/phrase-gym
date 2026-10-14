import asyncio
import json
from pathlib import Path
from uuid import UUID

import pytest
from sqlalchemy import select

from phrase_gym.contracts.content import DialogueInput, DialogueTurn
from phrase_gym.preparation import versions
from phrase_gym.preparation.jobs import digest
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.preparation.speech import SpeechAdapter, SpeechSettings
from phrase_gym.preparation.worker import PreparationWorker
from phrase_gym.tables import Target, TargetRevision
from tests.preparation.conftest import FakeProvider, FakeSpeech
from tests.preparation.test_publication import completed


def speech_boundary(media, monkeypatch):
    calls = []
    outage = {"partner": False}

    class Edge:
        def __init__(self, text, voice):
            self.text = text
            calls.append((text, voice))

        async def save(self, path):
            if outage["partner"] and self.text == "How are you?":
                raise OSError("Synthetic partner outage")
            Path(path).write_bytes(Path("tests/fixtures/media/tone.mp3").read_bytes())

    monkeypatch.setattr("phrase_gym.preparation.speech.edge_tts.Communicate", Edge)
    settings = SpeechSettings.model_validate(
        {"voices": {"en": [{"engine": "edge", "voice": "en-US-AriaNeural", "locale": "en-US"}]}},
    )
    return SpeechAdapter(media, settings), calls, outage


def partner_source(source):
    source.dialogues = [
        DialogueInput(
            source_id="exchange",
            title="Greeting",
            turns=[
                DialogueTurn(speaker="A", text="How are you?"),
                DialogueTurn(speaker="B", text=source.phrases[0].text, target_source_id="greeting"),
            ],
        ),
    ]


@pytest.mark.parametrize("upgrade", [False, True])
def test_partial_audio_retry_fences_versions_and_preserves_old_checkpoint(
    prepared_runtime, source, draft, monkeypatch, upgrade
):
    jobs, media, _, _ = prepared_runtime
    partner_source(source)
    speech, calls, outage = speech_boundary(media, monkeypatch)
    provider = FakeProvider(draft)
    outage["partner"] = True
    old = jobs.enqueue(source, {})
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(old.id))
    failed = jobs.get(old.id)
    assert failed.status == "failed"
    old_audio = failed.result["audio"]["target:greeting"]
    assert failed.result["audio_preparation_version"] == versions.SPEECH_VERSION
    assert len(calls) == 2 and len(provider.calls) == 2
    assert jobs.enqueue(source, {}).id == old.id
    outage["partner"] = False
    if upgrade:
        monkeypatch.setattr(versions, "SPEECH_VERSION", "synthetic-upgrade")
    jobs.retry(old.id)
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(old.id))
    if not upgrade:
        resumed = jobs.get(old.id)
        assert resumed.status == "awaiting_review"
        assert resumed.result["audio"]["target:greeting"] == old_audio
        assert len(calls) == 3 and len(provider.calls) == 2
        return
    refused = jobs.get(old.id)
    assert refused.status == "failed" and refused.error.startswith("speech_version_mismatch")
    assert refused.result == failed.result
    assert len(calls) == 2 and len(provider.calls) == 2
    new = jobs.enqueue(source, {})
    assert new.id != old.id and new.input_hash != old.input_hash
    assert jobs.enqueue(source, {}).id == new.id
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(new.id))
    current = jobs.get(new.id)
    assert current.status == "awaiting_review"
    assert current.result["audio"]["target:greeting"] != old_audio
    assert current.result["audio_preparation_version"] == "synthetic-upgrade"
    assert (
        media.get(UUID(current.result["audio"]["target:greeting"])).metadata.preparation_version
        == "synthetic-upgrade"
    )
    assert jobs.get(old.id).result == failed.result


def test_ready_job_is_historical_and_new_speech_version_has_new_identity(
    prepared_runtime, source, draft, monkeypatch
):
    jobs, _, imports, _ = prepared_runtime
    old, review = completed(prepared_runtime, source, draft)
    ready = publish_reviewed(jobs, imports, old.id, review)
    assert jobs.enqueue(source, {}).id == ready.id
    with jobs.db.read() as transaction:
        target = transaction.scalar(select(Target))
        old_payload = transaction.get(TargetRevision, (target.id, 1)).payload
    monkeypatch.setattr(versions, "SPEECH_VERSION", "synthetic-upgrade")
    new = jobs.enqueue(source, {})
    assert new.id != ready.id and new.input_hash != ready.input_hash
    assert jobs.get(old.id) == ready
    assert publish_reviewed(jobs, imports, old.id, review) == ready
    with jobs.db.read() as transaction:
        assert transaction.get(Target, target.id).current_revision == 1
        assert transaction.get(TargetRevision, (target.id, 1)).payload == old_payload


@pytest.mark.parametrize("retained_audio", [False, True])
def test_legacy_payload_without_speech_version_refuses_resume_without_calls(
    prepared_runtime, source, draft, retained_audio
):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})
    with jobs.db.write() as transaction:
        row = jobs.row(transaction, job.id)
        payload = json.loads(row.payload)
        payload.pop("speech_version")
        row.payload = json.dumps(payload)
        row.input_hash = digest(row.payload)
        row.result = json.dumps({"audio": {"target:greeting": "old-id"}} if retained_audio else {})
    original = jobs.get(job.id)
    provider, speech = FakeProvider(draft), FakeSpeech(media)
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(job.id))
    refused = jobs.get(job.id)
    assert refused.status == "failed" and refused.error.startswith("speech_version_mismatch")
    assert refused.result == original.result
    assert provider.calls == [] and speech.calls == 0
    assert jobs.enqueue(source, {}).id != job.id


@pytest.mark.parametrize("checkpoint_version", [None, "older-version"])
def test_missing_or_mismatched_audio_checkpoint_version_is_withheld(
    prepared_runtime, source, draft, checkpoint_version
):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})
    retained = {"audio": {"target:greeting": "old-id"}}
    if checkpoint_version:
        retained["audio_preparation_version"] = checkpoint_version
    with jobs.db.write() as transaction:
        jobs.row(transaction, job.id).result = json.dumps(retained)
    provider, speech = FakeProvider(draft), FakeSpeech(media)
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(job.id))
    refused = jobs.get(job.id)
    assert refused.status == "failed"
    assert refused.error.startswith("speech_checkpoint_version_mismatch")
    assert refused.result == retained and provider.calls == [] and speech.calls == 0
    jobs.retry(job.id, regenerate=True)
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(job.id))
    recovered = jobs.get(job.id)
    assert recovered.status == "awaiting_review"
    assert recovered.result["previous_runs"] == [retained]


@pytest.mark.parametrize("change", ["prompt", "provider"])
def test_unrelated_llm_change_creates_job_but_reuses_identical_speech_media(
    prepared_runtime, source, draft, monkeypatch, change
):
    jobs, media, _, _ = prepared_runtime
    speech, calls, _ = speech_boundary(media, monkeypatch)
    old = jobs.enqueue(source, {"provider": "first"})
    asyncio.run(PreparationWorker(jobs, FakeProvider(draft), speech).once(old.id))
    original = jobs.get(old.id).result["audio"]["target:greeting"]
    settings = {"provider": "first"}
    if change == "prompt":
        monkeypatch.setattr(versions, "PROMPT_VERSION", "synthetic-prompt-upgrade")
    else:
        settings["provider"] = "second"
    new = jobs.enqueue(source, settings)
    assert new.id != old.id
    provider = FakeProvider(draft)
    asyncio.run(PreparationWorker(jobs, provider, speech).once(new.id))
    current = jobs.get(new.id)
    assert current.status == "awaiting_review"
    assert current.result["audio"]["target:greeting"] == original
    assert len(calls) == 1 and len(provider.calls) == 2
