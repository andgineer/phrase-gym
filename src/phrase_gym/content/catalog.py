import json
from collections.abc import Callable
from typing import cast
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.base import Language
from phrase_gym.contracts.content import DialogueInput, TargetInput
from phrase_gym.contracts.library import (
    Dialogue,
    DialoguePage,
    Lesson,
    LessonPage,
    Phrase,
    PhrasePage,
)
from phrase_gym.errors import DomainError
from phrase_gym.storage import Database
from phrase_gym.tables import DialogueRow, LessonRow, MediaRow, Membership, Target, TargetRevision


class LibraryService:
    def __init__(self, database: Database, media_available: Callable[[MediaRow], bool]):
        self.db = database
        self.media_available = media_available

    @staticmethod
    def lesson_model(session: Session, row: LessonRow) -> Lesson:
        members = session.scalars(select(Membership).where(Membership.lesson_id == row.id)).all()
        return Lesson(
            id=UUID(row.id),
            language=cast(Language, row.language),
            title=row.title,
            phrase_count=len(members),
        )

    def lessons(self, language: Language, query: str, offset: int, limit: int) -> LessonPage:
        with self.db.read() as session:
            rows = session.scalars(
                select(LessonRow)
                .where(LessonRow.language == language)
                .order_by(LessonRow.title, LessonRow.id),
            ).all()
            matches = [r for r in rows if query.casefold() in r.title.casefold()]
            return LessonPage(
                items=[self.lesson_model(session, r) for r in matches[offset : offset + limit]],
                total=len(matches),
                offset=offset,
                limit=limit,
            )

    def phrase_model(
        self,
        session: Session,
        target: Target,
        revision: int | None = None,
    ) -> Phrase:
        row = session.get(TargetRevision, (target.id, revision or target.current_revision))
        if row is None:
            raise DomainError("Content revision not found", 404)
        content = TargetInput.model_validate_json(row.payload)
        lessons = session.scalars(
            select(LessonRow)
            .join(Membership)
            .where(Membership.target_id == target.id)
            .order_by(LessonRow.title, LessonRow.id),
        ).all()
        audio = (
            session.get(MediaRow, str(content.reference_audio_id))
            if content.reference_audio_id
            else None
        )
        ready = audio is not None and self.media_available(audio)
        cue_audio_ready = all(
            cue.audio_id is None
            or (
                (asset := session.get(MediaRow, str(cue.audio_id))) is not None
                and self.media_available(asset)
            )
            for cue in content.cues
        )
        return Phrase(
            id=UUID(target.id),
            revision=row.revision,
            language=cast(Language, target.language),
            content=content,
            lessons=[self.lesson_model(session, lesson) for lesson in lessons],
            audio_status="ready" if ready else "missing",
            retrieval_ready=bool(content.cues) and ready and cue_audio_ready,
        )

    def phrase(self, target_id: UUID, language: Language, revision: int | None = None) -> Phrase:
        with self.db.read() as session:
            target = session.get(Target, str(target_id))
            if target is None or target.language != language:
                raise DomainError("Phrase not found in this learning language", 404)
            return self.phrase_model(session, target, revision)

    def phrases(
        self,
        language: Language,
        query: str,
        offset: int,
        limit: int,
        lesson_id: UUID | None = None,
    ) -> PhrasePage:
        with self.db.read() as session:
            statement = select(Target).where(Target.language == language).order_by(Target.id)
            if lesson_id:
                lesson = session.get(LessonRow, str(lesson_id))
                if not lesson or lesson.language != language:
                    raise DomainError("Lesson not found in this learning language", 404)
                statement = (
                    statement.join(Membership)
                    .where(Membership.lesson_id == str(lesson_id))
                    .order_by(None)
                    .order_by(Membership.position)
                )
            results = [self.phrase_model(session, target) for target in session.scalars(statement)]
            search = query.casefold().strip()
            results = [
                phrase
                for phrase in results
                if search
                in " ".join(
                    [phrase.content.text, phrase.content.purpose, *phrase.content.concepts],
                ).casefold()
            ]
            return PhrasePage(
                items=results[offset : offset + limit],
                total=len(results),
                offset=offset,
                limit=limit,
            )

    @staticmethod
    def dialogue_model(row: DialogueRow) -> Dialogue:
        bindings = json.loads(row.target_ids)
        return Dialogue(
            id=UUID(row.id),
            revision=row.revision,
            language=cast(Language, row.language),
            content=DialogueInput.model_validate_json(row.payload),
            target_ids=bindings["ids"],
            target_revisions=bindings["revisions"],
        )

    def dialogues(self, language: Language, offset: int, limit: int) -> DialoguePage:
        with self.db.read() as session:
            rows = session.scalars(
                select(DialogueRow)
                .where(DialogueRow.language == language)
                .order_by(DialogueRow.source_id, DialogueRow.revision),
            ).all()
            latest = {row.id: row for row in rows}
            values = list(latest.values())
            return DialoguePage(
                items=[self.dialogue_model(row) for row in values[offset : offset + limit]],
                total=len(values),
                offset=offset,
                limit=limit,
            )

    def dialogue(
        self,
        dialogue_id: UUID,
        language: Language,
        revision: int | None = None,
    ) -> Dialogue:
        with self.db.read() as session:
            statement = select(DialogueRow).where(
                DialogueRow.id == str(dialogue_id),
                DialogueRow.language == language,
            )
            if revision is not None:
                statement = statement.where(DialogueRow.revision == revision)
            row = session.scalar(statement.order_by(DialogueRow.revision.desc()))
            if row is None:
                raise DomainError("Dialogue not found in this learning language", 404)
            return self.dialogue_model(row)
