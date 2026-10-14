import asyncio
from datetime import timedelta

from phrase_gym.preparation.worker import PreparationWorker
from tests.preparation.conftest import FakeProvider, FakeSpeech


def test_audio_outage_retains_draft_and_does_not_repeat_paid_calls(prepared_runtime, source, draft):
    jobs, media, _, clock = prepared_runtime
    job = jobs.enqueue(source, {})
    provider = FakeProvider(draft)
    speech = FakeSpeech(media)
    speech.fail = True
    worker = PreparationWorker(jobs, provider, speech)
    assert asyncio.run(worker.once())
    failed = jobs.get(job.id)
    assert failed.status == "failed"
    assert "generation" in failed.result and "model_review" in failed.result
    speech.fail = False
    clock[0] += timedelta(minutes=2)
    assert asyncio.run(worker.once())
    completed = jobs.get(job.id)
    assert completed.status == "awaiting_review"
    assert len(provider.calls) == 2
    assert completed.result["package"]["targets"][0]["reference_audio_id"]
    assert not asyncio.run(worker.once())


def test_malformed_output_is_retained_without_publication(prepared_runtime, source, draft):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})

    class Malformed:
        async def ask(self, *_):
            return {"text": "{broken json", "model": "fake"}

    asyncio.run(PreparationWorker(jobs, Malformed(), FakeSpeech(media)).once())
    failed = jobs.get(job.id)
    assert failed.status == "failed" and failed.result["generation"]["text"] == "{broken json"
    assert failed.published_import_id is None


def test_explicit_regeneration_archives_malformed_answers(prepared_runtime, source, draft):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})

    class Malformed:
        async def ask(self, *_):
            return {"text": "{bad json", "model": "fake"}

    asyncio.run(PreparationWorker(jobs, Malformed(), FakeSpeech(media)).once())
    jobs.retry(job.id, regenerate=True)
    asyncio.run(PreparationWorker(jobs, FakeProvider(draft), FakeSpeech(media)).once())
    result = jobs.get(job.id)
    assert result.status == "awaiting_review"
    assert result.result["previous_runs"][0]["generation"]["text"] == "{bad json"
