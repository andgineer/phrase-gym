import hashlib
import json

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.serialization import canonical_json
from phrase_gym.storage import DialogueRow, LessonRow, Membership, SourceItem, Target


def fingerprint(text: str) -> str:
    return hashlib.sha256(canonical_json(text).encode()).hexdigest()


def current_revision(session: Session, target_id: str) -> int:
    target = session.get(Target, target_id)
    assert target is not None
    return target.current_revision


def content_version(session: Session, namespace: str) -> str:
    sources = session.scalars(
        select(SourceItem).where(SourceItem.namespace == namespace).order_by(SourceItem.source_id),
    ).all()
    lessons = session.scalars(
        select(LessonRow).where(LessonRow.namespace == namespace).order_by(LessonRow.source_id),
    ).all()
    dialogues = session.scalars(
        select(DialogueRow)
        .where(DialogueRow.namespace == namespace)
        .order_by(DialogueRow.source_id, DialogueRow.revision),
    ).all()
    snapshot = {
        "targets": [
            (
                row.source_id,
                row.target_id,
                current_revision(session, row.target_id),
            )
            for row in sources
        ],
        "lessons": [
            (
                row.id,
                row.title,
                [
                    (m.target_id, m.position)
                    for m in session.scalars(
                        select(Membership)
                        .where(Membership.lesson_id == row.id)
                        .order_by(Membership.position),
                    )
                ],
            )
            for row in lessons
        ],
        "dialogues": [(row.id, row.revision) for row in dialogues],
    }
    return fingerprint(json.dumps(snapshot, sort_keys=True))
