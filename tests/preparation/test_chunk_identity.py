from pathlib import Path
from uuid import UUID
import json

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import select

from phrase_gym.api.app import create_app
from phrase_gym.config import Config
from phrase_gym.errors import DomainError
from phrase_gym.contracts.content import Chunk, ManualPackage, TargetInput
from phrase_gym.practice.tables import ChunkHistoryRow
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.preparation.jobs import digest
from phrase_gym.preparation.revisions import revise_draft
from phrase_gym.preparation.validation import assemble
from phrase_gym.tables import Target, TargetRevision
from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply
from tests.integration.session_helpers import start
from tests.preparation.test_publication import completed


def published(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    job, review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, job.id, review)
    with jobs.db.read() as transaction:
        target = transaction.scalar(select(Target))
        return target.id, TargetInput.model_validate_json(
            transaction.get(TargetRevision, (target.id, target.current_revision)).payload
        )


def seed_recall(jobs, target_id, chunk_id="a"):
    with jobs.db.write() as transaction:
        transaction.add(
            ChunkHistoryRow(
                target_id=target_id,
                chunk_id=chunk_id,
                presentations=10,
                recalls=10,
                failures=0,
                last_presented=NOW.isoformat(),
            )
        )


def renamed(draft, mapping):
    changed = draft.model_copy(deep=True)
    phrase = changed.phrases[0]
    for chunk in phrase.chunks:
        chunk.id = mapping.get(chunk.id, chunk.id)
    phrase.required_chunks = [
        mapping.get(chunk_id, chunk_id) for chunk_id in phrase.required_chunks
    ]
    for candidate in phrase.candidates:
        candidate.visible_chunks = [
            mapping.get(chunk_id, chunk_id) for chunk_id in candidate.visible_chunks
        ]
    phrase.uncovered_chunks = {
        mapping.get(chunk_id, chunk_id): reason
        for chunk_id, reason in phrase.uncovered_chunks.items()
    }
    phrase.slots = {
        mapping.get(chunk_id, chunk_id): values for chunk_id, values in phrase.slots.items()
    }
    return changed


@pytest.mark.parametrize("mapping", [{"a": "renamed-a", "c": "renamed-c"}, {"a": "c", "c": "a"}])
def test_generated_id_rename_or_swap_preserves_only_matching_material(
    prepared_runtime, source, draft, mapping
):
    jobs, _, imports, _ = prepared_runtime
    target_id, _ = published(prepared_runtime, source, draft)
    seed_recall(jobs, target_id)
    source.source_revision = "renamed"
    changed = renamed(draft, mapping)
    changed.phrases[0].concepts.append("regenerated")
    job, review = completed(prepared_runtime, source, changed)
    assert job.result["chunk_id_mapping"]["greeting"][mapping["a"]] == "a"
    assert job.result["chunk_id_mapping"]["greeting"][mapping["c"]] == "c"
    publish_reviewed(jobs, imports, job.id, review)
    with jobs.db.read() as transaction:
        content = TargetInput.model_validate_json(
            transaction.get(TargetRevision, (target_id, 2)).payload
        )
        assert {c.id: c.text for c in content.chunks} == {
            "a": "Not too bad, ",
            "b": "thanks. ",
            "c": "And yours?",
        }
        assert transaction.get(ChunkHistoryRow, (target_id, "a")).recalls == 10
        assert transaction.get(ChunkHistoryRow, (target_id, "c")) is None


def merged(draft):
    changed = draft.model_copy(deep=True)
    phrase = changed.phrases[0]
    phrase.chunks = [Chunk(id="a", text="Not too bad, thanks. "), Chunk(id="c", text="And yours?")]
    phrase.required_chunks = ["a", "c"]
    phrase.uncovered_chunks = {}
    phrase.candidates[0].prompt = "_____ And yours?"
    phrase.candidates[0].visible_chunks = ["c"]
    phrase.candidates[1].visible_chunks = ["a"]
    return changed


def test_changed_material_has_distinct_history_and_late_pinned_events_stay_original(
    prepared_runtime, source, draft
):
    jobs, _, imports, clock = prepared_runtime
    target_id, _ = published(prepared_runtime, source, draft)
    config = Config(Path(jobs.db.engine.url.database).parent)
    with TestClient(create_app(config, clock=lambda: clock[0])) as client:
        old = start(client).json()
        pinned = old["manifest"]
        assert pinned["targets"][0]["hidden_chunks"] == ["a"]
        seed_recall(jobs, target_id)
        source.source_revision = "merged"
        new_job, approval = completed(prepared_runtime, source, merged(draft))
        replacement = new_job.result["chunk_id_mapping"]["greeting"]["a"]
        assert replacement != "a" and replacement.startswith("material.")
        publish_reviewed(jobs, imports, new_job.id, approval)
        _, result = apply(client, old, "present")
        _, result = apply(client, result["session"], "rate", rating="recalled")
        assert result["session"]["attempts"][0]["eligibility_reason"] == "stale_content"
        assert client.get(f"/api/v1/sessions/{old['id']}").json()["manifest"] == pinned
        with jobs.db.read() as transaction:
            assert transaction.get(ChunkHistoryRow, (target_id, replacement)) is None
            assert transaction.get(ChunkHistoryRow, (target_id, "a")).presentations == 11
            assert transaction.get(ChunkHistoryRow, (target_id, "a")).recalls == 10
        new = start(client).json()
        assert new["manifest"]["targets"][0]["hidden_chunks"] == [replacement]
        _, result = apply(client, new, "present")
        _, result = apply(client, result["session"], "rate", rating="recalled")
    with jobs.db.read() as transaction:
        assert transaction.get(ChunkHistoryRow, (target_id, replacement)).recalls == 1
        assert transaction.get(ChunkHistoryRow, (target_id, "a")).recalls == 10


def test_ambiguous_id_from_all_retained_revisions_is_quarantined(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    target_id, _ = published(prepared_runtime, source, draft)
    seed_recall(jobs, target_id)
    with jobs.db.read() as transaction:
        original = TargetInput.model_validate_json(
            transaction.get(TargetRevision, (target_id, 1)).payload
        )
    corrupt = original.model_copy(deep=True)
    for chunk in corrupt.chunks:
        if chunk.id == "a":
            chunk.id = "c"
        elif chunk.id == "c":
            chunk.id = "a"
    corrupt.required_chunks = ["c", "b", "a"]
    for cue in corrupt.cues:
        cue.visible_chunks = [
            {"a": "c", "c": "a"}.get(chunk_id, chunk_id) for chunk_id in cue.visible_chunks
        ]
    package = ManualPackage(
        namespace=source.namespace,
        language=source.language,
        source_revision="legacy-collision",
        targets=[corrupt],
    )
    staged = imports.stage(package)
    imports.publish(staged.id)
    # Remove both collided IDs from the current revision: the older bindings still matter.
    cleaned = renamed(draft, {"a": "clean-prefix", "c": "clean-ending"})
    source.source_revision = "cleaned"
    cleaned_job, approval = completed(prepared_runtime, source, cleaned)
    publish_reviewed(jobs, imports, cleaned_job.id, approval)
    source.source_revision = "reused-legacy-name"
    regenerated, review = completed(prepared_runtime, source, draft)
    mapping = regenerated.result["chunk_id_mapping"]["greeting"]
    assert mapping["a"] != "a" and mapping["c"] != "c"
    publish_reviewed(jobs, imports, regenerated.id, review)
    with jobs.db.read() as transaction:
        assert transaction.get(ChunkHistoryRow, (target_id, "a")).recalls == 10
        assert transaction.get(ChunkHistoryRow, (target_id, mapping["a"])) is None


def test_pre_normalization_approval_requires_correction_and_new_hash(
    prepared_runtime, source, draft
):
    jobs, _, imports, _ = prepared_runtime
    published(prepared_runtime, source, draft)
    source.source_revision = "legacy-approval"
    raw = renamed(draft, {"a": "renamed-a", "c": "renamed-c"})
    pending, approval = completed(prepared_runtime, source, raw)
    legacy = pending.result
    legacy["draft"] = raw.model_dump(mode="json")
    raw_package = assemble(source, raw)
    raw_package.targets[0].reference_audio_id = UUID(
        pending.result["package"]["targets"][0]["reference_audio_id"],
    )
    legacy["package"] = raw_package.model_dump(mode="json")
    legacy.pop("chunk_id_mapping")
    with jobs.db.write() as transaction:
        jobs.row(transaction, pending.id).result = json.dumps(legacy)
    old_approval = approval.model_copy(update={"draft_hash": digest(json.dumps(legacy))})
    with pytest.raises(DomainError, match="Chunk identities require correction"):
        publish_reviewed(jobs, imports, pending.id, old_approval)
    assert jobs.get(pending.id).result == legacy
    corrected = revise_draft(
        jobs, pending.id, raw, "Test operator", "Normalize stored material identities"
    )
    with pytest.raises(DomainError, match="hashes"):
        publish_reviewed(jobs, imports, pending.id, old_approval)
    fresh = approval.model_copy(update={"draft_hash": digest(json.dumps(corrected.result))})
    assert publish_reviewed(jobs, imports, pending.id, fresh).status == "ready"
