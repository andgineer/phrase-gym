from collections.abc import Callable
from datetime import datetime
from uuid import UUID

from phrase_gym.contracts.base import Language
from phrase_gym.contracts.readiness import DueSummary
from phrase_gym.practice.cues import select_cue
from phrase_gym.sessions.planning import SessionPlanner
from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow


class ReadinessService:
    def __init__(self, database: Database, planner: SessionPlanner, clock: Callable[[], datetime]):
        self.db, self.planner, self.clock = database, planner, clock

    def summary(self, language: Language, lesson_id: UUID | None) -> DueSummary:
        result = DueSummary(due_count=0, new_count=0, preparing_count=0, next_due_at=None)
        now = self.clock()
        with self.db.read() as transaction:
            statement, _dialogue = self.planner.scope(transaction, language, lesson_id, None)
            for target in transaction.scalars(statement):
                phrase = self.planner.library.phrase_model(transaction, target)
                cue = select_cue(transaction, target.id, phrase.revision, phrase.content)
                if not phrase.retrieval_ready or cue is None:
                    result.preparing_count += 1
                    continue
                schedule = transaction.get(ScheduleRow, target.id)
                if schedule is None:
                    result.new_count += 1
                elif datetime.fromisoformat(schedule.due_at) <= now:
                    result.due_count += 1
                else:
                    due = datetime.fromisoformat(schedule.due_at)
                    result.next_due_at = min(result.next_due_at, due) if result.next_due_at else due
        return result
