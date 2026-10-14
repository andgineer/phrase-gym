"""Fence every published package identity against its intake-time state."""

import json

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.preparation import SourcePackage
from phrase_gym.tables import DialogueRow, LessonRow, Membership, SourceItem, Target, TargetRevision


def target_binding(transaction: Session, target_id: str) -> dict | None:
    target = transaction.get(Target, target_id)
    if target is None:
        return None
    revision = transaction.get(TargetRevision, (target.id, target.current_revision))
    return {
        "id": target.id,
        "revision": target.current_revision,
        "fingerprint": revision.fingerprint if revision else None,
    }


def source_bindings(transaction: Session, source: SourcePackage) -> str:
    targets: dict[str, dict | None] = {}
    for phrase in source.phrases:
        item = transaction.get(SourceItem, (source.namespace, phrase.source_id))
        targets[phrase.source_id] = (
            {
                "source_revision": item.source_revision,
                "target": target_binding(transaction, item.target_id),
            }
            if item
            else None
        )
    snapshot = {
        "schema_version": 2,
        "targets": targets,
        "lessons": {
            lesson.source_id: lesson_binding(transaction, source.namespace, lesson.source_id)
            for lesson in source.lessons
        },
        "dialogues": {
            dialogue.source_id: dialogue_binding(transaction, source.namespace, dialogue.source_id)
            for dialogue in source.dialogues
        },
    }
    return json.dumps(snapshot, sort_keys=True)


def lesson_binding(transaction: Session, namespace: str, source_id: str) -> dict | None:
    row = transaction.scalar(
        select(LessonRow).where(
            LessonRow.namespace == namespace,
            LessonRow.source_id == source_id,
        ),
    )
    if row is None:
        return None
    members = transaction.scalars(
        select(Membership)
        .where(Membership.lesson_id == row.id)
        .order_by(Membership.position, Membership.target_id),
    ).all()
    return {
        "id": row.id,
        "language": row.language,
        "title": row.title,
        "members": [
            {"position": member.position, "target": target_binding(transaction, member.target_id)}
            for member in members
        ],
    }


def dialogue_binding(transaction: Session, namespace: str, source_id: str) -> dict | None:
    row = transaction.scalar(
        select(DialogueRow)
        .where(
            DialogueRow.namespace == namespace,
            DialogueRow.source_id == source_id,
        )
        .order_by(DialogueRow.revision.desc()),
    )
    if row is None:
        return None
    bindings = json.loads(row.target_ids)
    return {
        "id": row.id,
        "revision": row.revision,
        "language": row.language,
        "payload": json.loads(row.payload),
        "bindings": bindings,
        "current_targets": {
            target_id: target_binding(transaction, target_id)
            for target_id in bindings["ids"].values()
        },
    }
