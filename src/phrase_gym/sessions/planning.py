from datetime import datetime
from uuid import UUID, uuid4

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.content.catalog import LibraryService
from phrase_gym.contracts.base import Language
from phrase_gym.contracts.library import Dialogue
from phrase_gym.contracts.sessions import PlannedTarget, SessionManifest
from phrase_gym.errors import DomainError
from phrase_gym.practice.cues import select_cue
from phrase_gym.practice.policy import POLICY_VERSION
from phrase_gym.tables import DialogueRow, LessonRow, MediaRow, Membership, ScheduleRow, Target

MAX_DUE_TARGETS = 20
MAX_NEW_TARGETS = 5


class SessionPlanner:
    def __init__(self, library: LibraryService):
        self.library = library

    def scope(
        self,
        transaction: Session,
        language: Language,
        lesson_id: UUID | None,
        dialogue_id: UUID | None,
    ):
        if lesson_id and dialogue_id:
            raise DomainError("Choose a lesson or a dialogue, not both", 422)
        statement = select(Target).where(Target.language == language).order_by(Target.id)
        if lesson_id:
            lesson = transaction.get(LessonRow, str(lesson_id))
            if lesson is None or lesson.language != language:
                raise DomainError("Lesson not found in this learning language", 404)
            statement = statement.join(Membership).where(Membership.lesson_id == str(lesson_id))
        dialogue = None
        if dialogue_id:
            row = transaction.scalar(
                select(DialogueRow)
                .where(
                    DialogueRow.id == str(dialogue_id),
                    DialogueRow.language == language,
                )
                .order_by(DialogueRow.revision.desc()),
            )
            if row is None:
                raise DomainError("Dialogue not found in this learning language", 404)
            dialogue = self.library.dialogue_model(row)
            statement = statement.where(
                Target.id.in_([str(i) for i in dialogue.target_ids.values()]),
            )
        return statement, dialogue

    def planned(
        self,
        transaction: Session,
        target: Target,
        dialogue: Dialogue | None,
        now: datetime,
    ) -> PlannedTarget | None:
        schedule = transaction.get(ScheduleRow, target.id)
        if not dialogue and schedule and datetime.fromisoformat(schedule.due_at) > now:
            return None
        revisions = (
            {
                str(dialogue.target_ids[key]): value
                for key, value in dialogue.target_revisions.items()
            }
            if dialogue
            else {}
        )
        phrase = self.library.phrase_model(transaction, target, revisions.get(target.id))
        if not phrase.retrieval_ready:
            return None
        cue = select_cue(transaction, target.id, phrase.revision, phrase.content)
        if cue is None:
            return None
        return PlannedTarget(
            phrase=phrase,
            cue_id=cue.id,
            hidden_chunks=[
                c for c in phrase.content.required_chunks if c not in cue.visible_chunks
            ],
            review_group_id=uuid4(),
            schedule_version=schedule.version if schedule else 0,
            eligibility="due" if schedule else "new",
        )

    def required_media(
        self,
        transaction: Session,
        items: list[PlannedTarget],
        dialogue: Dialogue | None,
    ) -> list[UUID]:
        media = {
            i.phrase.content.reference_audio_id
            for i in items
            if i.phrase.content.reference_audio_id
        }
        for item in items:
            cue = next(c for c in item.phrase.content.cues if c.id == item.cue_id)
            if cue.audio_id:
                media.add(cue.audio_id)
        if dialogue:
            for turn in dialogue.content.turns:
                if turn.audio_id:
                    asset = transaction.get(MediaRow, str(turn.audio_id))
                    if not asset or not self.library.media_available(asset):
                        raise DomainError("Dialogue partner audio is not prepared", 409)
                    media.add(turn.audio_id)
        return sorted(media, key=str)

    def plan(
        self,
        transaction: Session,
        language: Language,
        now: datetime,
        lesson_id: UUID | None = None,
        dialogue_id: UUID | None = None,
    ) -> SessionManifest:
        statement, dialogue = self.scope(transaction, language, lesson_id, dialogue_id)
        due: list[tuple[datetime, PlannedTarget]] = []
        new: list[PlannedTarget] = []
        for target in transaction.scalars(statement):
            item = self.planned(transaction, target, dialogue, now)
            if item is None:
                continue
            schedule = transaction.get(ScheduleRow, target.id)
            if schedule:
                due.append((datetime.fromisoformat(schedule.due_at), item))
            else:
                new.append(item)
        ordered = [item for _, item in sorted(due, key=lambda pair: pair[0])]
        due_items = ordered[:MAX_DUE_TARGETS]
        new_budget = min(MAX_NEW_TARGETS, MAX_DUE_TARGETS - len(due_items))
        selected = ordered + new if dialogue else due_items + new[:new_budget]
        return SessionManifest(
            language=language,
            policy_version=POLICY_VERSION,
            lesson_id=lesson_id,
            targets=selected,
            required_media_ids=self.required_media(transaction, selected, dialogue),
            dialogue=dialogue,
        )
