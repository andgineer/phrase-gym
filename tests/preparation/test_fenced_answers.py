import asyncio
from datetime import timedelta

import pytest

from phrase_gym.preparation.worker import PreparationWorker, parse_answer
from tests.preparation.conftest import FakeProvider, FakeSpeech


@pytest.mark.parametrize("answer", ["```json", "```", "```json\n", "```json\n{}"])
def test_truncated_fence_is_a_recoverable_failure_with_regeneration(
    prepared_runtime, source, draft, answer
):
    jobs, media, _, clock = prepared_runtime
    job = jobs.enqueue(source, {})

    class Truncated:
        async def ask(self, *_):
            return {"text": answer, "model": "fake"}

    worker = PreparationWorker(jobs, Truncated(), FakeSpeech(media))
    assert asyncio.run(worker.once(job.id))
    failed = jobs.get(job.id)
    assert failed.status == "failed" and failed.error == "draft_conformance_failure"
    assert failed.result["generation"]["text"] == answer
    clock[0] += timedelta(minutes=2)
    assert asyncio.run(worker.once(job.id))
    assert jobs.get(job.id).status == "failed"
    jobs.retry(job.id, regenerate=True)
    assert asyncio.run(PreparationWorker(jobs, FakeProvider(draft), FakeSpeech(media)).once(job.id))
    recovered = jobs.get(job.id)
    assert recovered.status == "awaiting_review"
    assert recovered.result["previous_runs"][0]["generation"]["text"] == answer


def test_complete_fence_and_unfenced_json_remain_supported():
    assert parse_answer('```json\n{"ok": true}\n```') == {"ok": True}
    assert parse_answer('{"ok": true}') == {"ok": True}
