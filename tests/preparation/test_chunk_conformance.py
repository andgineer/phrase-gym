import asyncio

import pytest

from phrase_gym.errors import DomainError
from phrase_gym.preparation.revisions import revise_draft
from phrase_gym.preparation.worker import PreparationWorker
from tests.preparation.conftest import FakeProvider, FakeSpeech
from tests.preparation.test_publication import completed


def malformed(draft, reference):
    invalid = draft.model_copy(deep=True)
    phrase = invalid.phrases[0]
    if reference == "visible":
        phrase.candidates[0].visible_chunks = ["missing"]
    elif reference == "slot":
        phrase.slots = {"missing": ["value"]}
    else:
        phrase.uncovered_chunks = {"missing": "Uncovered material"}
    return invalid


@pytest.mark.parametrize("reference", ["visible", "slot", "uncovered"])
def test_invalid_provider_references_fail_before_identity_remapping(
    prepared_runtime, source, draft, reference
):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})
    invalid = malformed(draft, reference)
    worker = PreparationWorker(jobs, FakeProvider(invalid), FakeSpeech(media))
    assert asyncio.run(worker.once(job.id))
    result = jobs.get(job.id)
    assert result.status == "failed" and result.error == "draft_conformance_failure"
    assert result.result["generation"]["text"] == invalid.model_dump_json()


@pytest.mark.parametrize("reference", ["visible", "slot", "uncovered"])
def test_invalid_operator_references_keep_original_draft_and_hash(
    prepared_runtime, source, draft, reference
):
    jobs, _, _, _ = prepared_runtime
    job, _ = completed(prepared_runtime, source, draft)
    with pytest.raises((DomainError, ValueError)):
        revise_draft(
            jobs, job.id, malformed(draft, reference), "Test operator", "Invalid reference"
        )
    assert jobs.get(job.id) == job
