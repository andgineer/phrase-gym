from datetime import datetime
from uuid import UUID, uuid4

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.content.catalog import LibraryService
from phrase_gym.contracts.base import Language
from phrase_gym.contracts.sessions import PlannedTarget, SessionManifest
from phrase_gym.errors import DomainError
from phrase_gym.storage import LessonRow, Membership, ScheduleRow, Target


class SessionPlanner:
    def __init__(self, library: LibraryService):
        self.library = library

    def plan(
        self,
        transaction: Session,
        language: Language,
        now: datetime,
        lesson_id: UUID | None = None,
    ) -> SessionManifest:
        statement = select(Target).where(Target.language == language).order_by(Target.id)
        if lesson_id:
            lesson = transaction.get(LessonRow, str(lesson_id))
            if lesson is None or lesson.language != language:
                raise DomainError("Lesson not found in this learning language", 404)
            statement = statement.join(Membership).where(Membership.lesson_id == str(lesson_id))
        targets = transaction.scalars(statement)
        due: list[tuple[datetime, PlannedTarget]] = []
        new: list[PlannedTarget] = []
        media: set[UUID] = set()
        for target in targets:
            schedule = transaction.get(ScheduleRow, target.id)
            if schedule is not None and datetime.fromisoformat(schedule.due_at) > now:
                continue
            phrase = self.library.phrase_model(transaction, target)
            if not phrase.retrieval_ready:
                continue
            cue = next(c for c in phrase.content.cues if c.reviewed)
            item = PlannedTarget(
                phrase=phrase,
                cue_id=cue.id,
                hidden_chunks=[
                    c for c in phrase.content.required_chunks if c not in cue.visible_chunks
                ],
                review_group_id=uuid4(),
                schedule_version=schedule.version if schedule else 0,
                eligibility="due" if schedule else "new",
            )
            if schedule:
                due.append((datetime.fromisoformat(schedule.due_at), item))
            else:
                new.append(item)
            if phrase.content.reference_audio_id:
                media.add(phrase.content.reference_audio_id)
            if cue.audio_id:
                media.add(cue.audio_id)
        return SessionManifest(
            language=language,
            lesson_id=lesson_id,
            targets=[item for _, item in sorted(due, key=lambda pair: pair[0])] + new,
            required_media_ids=sorted(media, key=str),
        )
