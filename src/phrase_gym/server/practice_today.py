from collections.abc import Callable
from dataclasses import replace
from datetime import UTC, datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.import_contracts import StrictModel
from phrase_gym.server.practice_selection import due_phrases, new_phrases
from phrase_gym.server.practice_state import finished_message
from phrase_gym.server.practice_views import load_view, require_lesson
from phrase_gym.server.records import ViewRecord
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.settings_store import read_settings
from phrase_gym.server.storage import Database


class PracticeToday(StrictModel):
    learning_date: str
    due_count: int
    new_count: int
    extra_available: bool
    message: str


class TodayPractice:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def read(self, lesson_id: str | None) -> PracticeToday:
        now = self.clock()
        with self.database.snapshot() as connection:
            require_lesson(connection, lesson_id)
            day = learning_day(connection, now)
            current = load_view(connection)
            view = ViewRecord(
                1,
                "",
                day,
                "daily",
                lesson_id,
                "[]",
                None,
                None,
                None,
                0,
                0,
                None,
                None,
                "{}",
            )
            if current is not None and current.learning_date == day:
                view = replace(current, lesson_id=lesson_id)
            progress = daily_progress(connection, day)
            due_left = max(progress.due_allowance - progress.due_completed, 0)
            new_left = max(progress.new_allowance - progress.new_completed, 0)
            due = due_phrases(connection, view, now)
            fresh = new_phrases(connection, view)
            due_count, new_count = min(len(due), due_left), min(len(fresh), new_left)
            # Daily resumes its regular card before drawing, even after a lesson/limit change.
            if (
                current
                and current.learning_date == day
                and current.phrase_id
                and not current.practice
            ):
                if current.kind == "recall":
                    due_count = max(
                        1,
                        min(
                            len(due) + int(not any(p.id == current.phrase_id for p in due)),
                            due_left,
                        ),
                    )
                else:
                    new_count = max(
                        1,
                        min(
                            len(fresh) + int(not any(p.id == current.phrase_id for p in fresh)),
                            new_left,
                        ),
                    )
            # Extra starts a new traversal, so its availability ignores the current skipped IDs.
            extra_view = replace(view, skipped_json="[]")
            settings = read_settings(connection)
            extra_due = due_phrases(connection, extra_view, now)
            extra_new = new_phrases(connection, extra_view)
            extra_available = bool(
                (settings.daily_due_limit and extra_due)
                or (settings.daily_new_limit and extra_new),
            )
            return PracticeToday(
                learning_date=day,
                due_count=due_count,
                new_count=new_count,
                extra_available=extra_available,
                message=finished_message(connection, view, now, progress),
            )
