import json
from collections.abc import Callable
from datetime import datetime
from uuid import UUID, uuid4

from sqlalchemy import delete, select
from sqlalchemy.orm import Session

from phrase_gym.content.identity import current_revision, fingerprint
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.imports import ImportChange
from phrase_gym.storage import (
    DialogueRow,
    LessonRow,
    Membership,
    SourceItem,
    Target,
    TargetRevision,
)


def publish_targets(
    session: Session,
    package: ManualPackage,
    changes: list[ImportChange],
    clock: Callable[[], datetime],
) -> dict[str, str]:
    target_ids: dict[str, str] = {}
    for content, change in zip(package.targets, changes, strict=True):
        target_id = str(change.target_id or uuid4())
        revision = (change.base_revision or 0) + 1
        target_ids[content.source_id] = target_id
        change.target_id = UUID(target_id)
        if change.action == "create":
            session.add(
                Target(
                    id=target_id,
                    language=package.language,
                    kind=content.kind,
                    meaning_key=content.meaning_key,
                    recall_expression=content.recall_expression,
                    current_revision=revision,
                ),
            )
            session.flush()
            session.add(
                SourceItem(
                    namespace=package.namespace,
                    source_id=content.source_id,
                    target_id=target_id,
                    source_revision=package.source_revision,
                ),
            )
        else:
            target = session.get(Target, target_id)
            source = session.get(SourceItem, (package.namespace, content.source_id))
            assert target is not None and source is not None
            source.source_revision = package.source_revision
            if change.action == "update":
                target.current_revision = revision
        if change.action != "unchanged":
            session.add(
                TargetRevision(
                    target_id=target_id,
                    revision=revision,
                    payload=content.model_dump_json(),
                    fingerprint=fingerprint(content.model_dump_json()),
                    created_at=clock().isoformat(),
                ),
            )
    return target_ids


def publish_lessons(
    session: Session,
    package: ManualPackage,
    target_ids: dict[str, str],
) -> None:
    for lesson in package.lessons:
        row = session.scalar(
            select(LessonRow).where(
                LessonRow.namespace == package.namespace,
                LessonRow.source_id == lesson.source_id,
            ),
        )
        if row is None:
            row = LessonRow(
                id=str(uuid4()),
                namespace=package.namespace,
                source_id=lesson.source_id,
                language=package.language,
                title=lesson.title,
            )
            session.add(row)
            session.flush()
        row.title = lesson.title
        session.execute(delete(Membership).where(Membership.lesson_id == row.id))
        for position, source_id in enumerate(lesson.target_source_ids):
            session.add(
                Membership(
                    lesson_id=row.id,
                    target_id=target_ids[source_id],
                    position=position,
                ),
            )


def publish_dialogues(
    session: Session,
    package: ManualPackage,
    target_ids: dict[str, str],
) -> None:
    for dialogue in package.dialogues:
        previous = session.scalar(
            select(DialogueRow)
            .where(
                DialogueRow.namespace == package.namespace,
                DialogueRow.source_id == dialogue.source_id,
            )
            .order_by(DialogueRow.revision.desc()),
        )
        linked_ids = {
            turn.target_source_id: target_ids[turn.target_source_id]
            for turn in dialogue.turns
            if turn.target_source_id
        }
        bindings = json.dumps(
            {
                "ids": linked_ids,
                "revisions": {
                    source: current_revision(session, target_id)
                    for source, target_id in linked_ids.items()
                },
            },
            sort_keys=True,
        )
        if (
            previous
            and previous.payload == dialogue.model_dump_json()
            and previous.target_ids == bindings
        ):
            continue
        session.add(
            DialogueRow(
                id=previous.id if previous else str(uuid4()),
                namespace=package.namespace,
                source_id=dialogue.source_id,
                revision=previous.revision + 1 if previous else 1,
                language=package.language,
                payload=dialogue.model_dump_json(),
                target_ids=bindings,
            ),
        )
