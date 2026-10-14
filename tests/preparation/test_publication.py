import asyncio
import json

import pytest
from sqlalchemy import select

from phrase_gym.contracts.preparation import HumanReview
from phrase_gym.errors import DomainError
from phrase_gym.practice.cues import select_cue
from phrase_gym.practice.tables import ChunkHistoryRow
from phrase_gym.preparation.jobs import digest
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.preparation.revisions import revise_draft
from phrase_gym.preparation.worker import PreparationWorker
from phrase_gym.contracts.content import TargetInput
from phrase_gym.tables import Target, TargetRevision, ScheduleRow
from tests.integration.helpers import NOW
from tests.preparation.conftest import FakeProvider, FakeSpeech


def completed(prepared_runtime, source, draft):
    jobs, media, _, _ = prepared_runtime
    job = jobs.enqueue(source, {})
    asyncio.run(PreparationWorker(jobs, FakeProvider(draft), FakeSpeech(media)).once())
    job = jobs.get(job.id)
    review = HumanReview(
        input_hash=job.input_hash,
        draft_hash=digest(json.dumps(job.result)),
        reviewer="Synthetic software-test approval; no real audio/listening claim",
        accepted_cues={"greeting": ["hide-a", "hide-c"]},
        help_reviewed=True,
        meaning_search_equivalents_reviewed=True,
        audio_reviewed=True,
        notes="Test only",
    )
    return job, review


def test_hash_gate_atomic_publication_and_existing_schedule(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    job, review = completed(prepared_runtime, source, draft)
    wrong = review.model_copy(update={"draft_hash": "0" * 64})
    with pytest.raises(DomainError, match="hashes"):
        publish_reviewed(jobs, imports, job.id, wrong)
    assert jobs.get(job.id).status == "awaiting_review"
    result = publish_reviewed(jobs, imports, job.id, review)
    assert result.status == "ready"
    assert publish_reviewed(jobs, imports, job.id, review) == result
    with jobs.db.write() as transaction:
        target = transaction.scalar(select(Target))
        target_id = target.id
        transaction.add(
            ScheduleRow(
                target_id=target.id,
                version=7,
                due_at=NOW.isoformat(),
                memory_state=None,
                policy_version=None,
            )
        )
    source.source_revision = "2"
    draft.phrases[0].concepts.append("new search concept")
    changed, changed_review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, changed.id, changed_review)
    with jobs.db.read() as transaction:
        targets = transaction.scalars(select(Target)).all()
        assert len(targets) == 1 and targets[0].id == target_id and targets[0].current_revision == 2
        assert transaction.get(ScheduleRow, target_id).version == 7
        old = transaction.get(TargetRevision, (target_id, 1))
        assert "new search concept" not in old.payload


def test_stale_source_change_does_not_overwrite_revision(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    job, review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, job.id, review)
    source.source_revision = "2"
    staged, staged_review = completed(prepared_runtime, source, draft)
    with jobs.db.write() as transaction:
        target = transaction.scalar(select(Target))
        old = transaction.get(TargetRevision, (target.id, target.current_revision))
        old.fingerprint = "concurrent change"
    with pytest.raises(DomainError, match="changed during"):
        publish_reviewed(jobs, imports, staged.id, staged_review)
    assert jobs.get(staged.id).status == "awaiting_review"


def test_generated_mask_rotation_and_revision_pinning(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    job, review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, job.id, review)
    with jobs.db.write() as transaction:
        target = transaction.scalar(select(Target))
        content = TargetInput.model_validate_json(
            transaction.get(TargetRevision, (target.id, 1)).payload
        )
        first = select_cue(transaction, target.id, 1, content)
        assert first.id == "hide-a"
        transaction.add(
            ChunkHistoryRow(
                target_id=target.id,
                chunk_id="a",
                presentations=1,
                recalls=0,
                failures=0,
                last_presented=NOW.isoformat(),
            )
        )
        transaction.flush()
        assert select_cue(transaction, target.id, 1, content).id == "hide-c"
        assert first.id == "hide-a"  # The selected session value remains pinned.


def test_operator_correction_retains_raw_evidence_and_invalidates_old_hash(
    prepared_runtime, source, draft
):
    jobs, _, imports, _ = prepared_runtime
    job, old_review = completed(prepared_runtime, source, draft)
    raw = job.result["generation"]["text"]
    draft.phrases[0].explanation.meaning = "Неплохо, спасибо. А у тебя как?"
    revised = revise_draft(jobs, job.id, draft, "Test operator", "Corrected native meaning")
    assert revised.result["generation"]["text"] == raw
    assert (
        revised.result["operator_revisions"][0]["previous_draft"]["phrases"][0]["explanation"][
            "meaning"
        ]
        != draft.phrases[0].explanation.meaning
    )
    with pytest.raises(DomainError, match="hashes"):
        publish_reviewed(jobs, imports, job.id, old_review)
    approval = old_review.model_copy(update={"draft_hash": digest(json.dumps(revised.result))})
    assert publish_reviewed(jobs, imports, job.id, approval).status == "ready"
