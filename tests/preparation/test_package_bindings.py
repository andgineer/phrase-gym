import json

import pytest
from sqlalchemy import select

from phrase_gym.contracts.content import DialogueInput, DialogueTurn, ManualPackage
from phrase_gym.errors import DomainError
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.tables import DialogueRow, LessonRow, Target
from tests.preparation.test_publication import completed


def with_dialogue(source):
    source.dialogues = [
        DialogueInput(
            source_id="exchange",
            title="Original exchange",
            turns=[
                DialogueTurn(speaker="A", text="How are you?"),
                DialogueTurn(speaker="B", text=source.phrases[0].text, target_source_id="greeting"),
            ],
        )
    ]


def manual_publish(imports, payload):
    staged = imports.stage(ManualPackage.model_validate(payload))
    assert not staged.errors
    imports.publish(staged.id)


@pytest.mark.parametrize(
    "change",
    ["lesson_title", "lesson_members", "dialogue_title", "dialogue_bindings", "source_revision"],
)
def test_pending_package_fences_all_existing_publication_identities(
    prepared_runtime, source, draft, change
):
    jobs, _, imports, _ = prepared_runtime
    with_dialogue(source)
    initial, approval = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, initial.id, approval)
    source.source_revision = "pending-2"
    pending, review = completed(prepared_runtime, source, draft)
    payload = initial.result["package"]
    if change == "lesson_title":
        payload["lessons"][0]["title"] = "New reviewed lesson title"
    elif change == "lesson_members":
        other = dict(payload["targets"][0], source_id="other", meaning_key="other")
        payload["targets"].append(other)
        payload["lessons"][0]["target_source_ids"].append("other")
    elif change == "dialogue_title":
        payload["dialogues"][0]["title"] = "New reviewed exchange"
    elif change == "source_revision":
        payload["source_revision"] = "New reviewed provenance revision"
    else:
        with jobs.db.write() as transaction:
            row = transaction.scalar(select(DialogueRow))
            bindings = json.loads(row.target_ids)
            bindings["revisions"]["greeting"] += 1
            row.target_ids = json.dumps(bindings)
    if change != "dialogue_bindings":
        manual_publish(imports, payload)
    with jobs.db.read() as transaction:
        assert (
            transaction.scalar(
                select(Target).where(Target.meaning_key == "greeting")
            ).current_revision
            == 1
        )
    with pytest.raises(DomainError, match="changed during"):
        publish_reviewed(jobs, imports, pending.id, review)
    assert jobs.get(pending.id).status == "awaiting_review"
    if change == "lesson_title":
        with jobs.db.read() as transaction:
            assert transaction.scalar(select(LessonRow)).title == "New reviewed lesson title"


@pytest.mark.parametrize("kind", ["lesson", "dialogue"])
def test_pending_package_fences_absent_identities_created_elsewhere(
    prepared_runtime, source, draft, kind
):
    jobs, _, imports, _ = prepared_runtime
    with_dialogue(source)
    initial_source = source.model_copy(deep=True)
    setattr(initial_source, "lessons" if kind == "lesson" else "dialogues", [])
    initial, approval = completed(prepared_runtime, initial_source, draft)
    publish_reviewed(jobs, imports, initial.id, approval)
    source.source_revision = "pending-create"
    pending, review = completed(prepared_runtime, source, draft)
    payload = pending.result["package"]
    payload["source_revision"] = initial_source.source_revision
    manual_publish(imports, payload)
    with pytest.raises(DomainError, match="changed during"):
        publish_reviewed(jobs, imports, pending.id, review)


def test_pending_lesson_fences_revision_of_preexisting_member_not_in_source(
    prepared_runtime, source, draft
):
    jobs, _, imports, _ = prepared_runtime
    initial, approval = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, initial.id, approval)
    payload = initial.result["package"]
    other = dict(payload["targets"][0], source_id="other", meaning_key="other")
    payload["targets"].append(other)
    payload["lessons"][0]["target_source_ids"].append("other")
    manual_publish(imports, payload)
    source.source_revision = "pending-member"
    pending, review = completed(prepared_runtime, source, draft)
    other["purpose"] = "A new reviewed purpose for the other member"
    manual_publish(imports, payload)
    with pytest.raises(DomainError, match="changed during"):
        publish_reviewed(jobs, imports, pending.id, review)
