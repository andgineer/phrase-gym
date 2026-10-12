from collections.abc import Callable
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.content.identity import content_version, fingerprint
from phrase_gym.contracts.base import Language
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.imports import ImportChange, ImportReport
from phrase_gym.contracts.media import MediaMetadata
from phrase_gym.storage import DialogueRow, LessonRow, MediaRow, SourceItem, Target, TargetRevision


class PackageValidator:
    def __init__(self, media_available: Callable[[MediaRow], bool]):
        self.media_available = media_available

    def report(self, session: Session, package: ManualPackage) -> ImportReport:
        changes: list[ImportChange] = []
        errors: list[str] = []
        for content in package.targets:
            source = session.get(SourceItem, (package.namespace, content.source_id))
            target = session.get(Target, source.target_id) if source else None
            action = "create"
            if target:
                if (
                    target.language,
                    target.kind,
                    target.meaning_key,
                    target.recall_expression,
                ) != (
                    package.language,
                    content.kind,
                    content.meaning_key,
                    content.recall_expression,
                ):
                    errors.append(
                        f"{content.source_id}: changed learning objective; use a new source ID",
                    )
                revision = session.get(TargetRevision, (target.id, target.current_revision))
                action = (
                    "unchanged"
                    if revision
                    and revision.fingerprint
                    == fingerprint(
                        content.model_dump_json(),
                    )
                    else "update"
                )
            changes.append(
                ImportChange(
                    source_id=content.source_id,
                    action=action,
                    target_id=UUID(target.id) if target else None,
                    base_revision=target.current_revision if target else None,
                ),
            )
            self.check_audio(
                session,
                content.reference_audio_id,
                package.language,
                content.text,
                "reference",
                errors,
                content.source_id,
            )
            for cue in content.cues:
                spoken = "".join(
                    chunk.text for chunk in content.chunks if chunk.id in cue.visible_chunks
                )
                self.check_audio(
                    session,
                    cue.audio_id,
                    package.language,
                    spoken,
                    "cue",
                    errors,
                    f"{content.source_id}/{cue.id}",
                )
        for lesson in package.lessons:
            existing = session.scalar(
                select(LessonRow).where(
                    LessonRow.namespace == package.namespace,
                    LessonRow.source_id == lesson.source_id,
                ),
            )
            if existing and existing.language != package.language:
                errors.append(
                    f"Lesson {lesson.source_id}: language changes require a new source ID",
                )
        for dialogue in package.dialogues:
            existing_dialogue = session.scalar(
                select(DialogueRow).where(
                    DialogueRow.namespace == package.namespace,
                    DialogueRow.source_id == dialogue.source_id,
                ),
            )
            if existing_dialogue and existing_dialogue.language != package.language:
                errors.append(
                    f"Dialogue {dialogue.source_id}: language changes require a new source ID",
                )
            for turn in dialogue.turns:
                self.check_audio(
                    session,
                    turn.audio_id,
                    package.language,
                    turn.text,
                    "reference" if turn.target_source_id else "partner",
                    errors,
                    dialogue.source_id,
                )
        return ImportReport(
            status="rejected" if errors else "validated",
            changes=changes,
            errors=errors,
            base_content_version=content_version(session, package.namespace),
        )

    def check_audio(  # noqa: PLR0913 - validates an asset against its content binding
        self,
        session: Session,
        audio_id: UUID | None,
        language: Language,
        text: str,
        role: str,
        errors: list[str],
        context: str,
    ) -> None:
        if audio_id is None:
            return
        media = session.get(MediaRow, str(audio_id))
        if not media or not self.media_available(media):
            errors.append(
                f"{context}: missing reference media {audio_id}; upload/recover it first",
            )
            return
        metadata = MediaMetadata.model_validate_json(media.metadata_json)
        if (metadata.language, metadata.text, metadata.role) != (language, text, role):
            errors.append(f"{context}: audio must match the exact language, text and role")
