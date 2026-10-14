import pytest

from phrase_gym.content.source_identity import source_changes
from phrase_gym.errors import DomainError
from phrase_gym.preparation.publication import publish_reviewed
from tests.preparation.test_publication import completed


def test_dry_run_create_edit_unchanged_and_meaning_conflict(prepared_runtime, source, draft):
    jobs, _, imports, _ = prepared_runtime
    with jobs.db.read() as transaction:
        assert source_changes(transaction, source)["changes"][0]["action"] == "create"
    job, review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, job.id, review)
    with jobs.db.read() as transaction:
        unchanged = source_changes(transaction, source)
        assert unchanged["changes"][0]["action"] == "unchanged"
        assert unchanged["changes"][0]["target_id"]
    source.phrases[0].purpose = "Edited source purpose"
    with jobs.db.read() as transaction:
        assert source_changes(transaction, source)["changes"][0]["action"] == "update"
    source.phrases[0].meaning_key = "different-meaning"
    with jobs.db.read() as transaction:
        assert source_changes(transaction, source)["identity_errors"]
    with pytest.raises(DomainError, match="new source ID"):
        jobs.enqueue(source, {})
