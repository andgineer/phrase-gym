import random
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.practice_contracts import PracticeState
from phrase_gym.server.practice_selection import advance
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import (
    load_view,
    require_lesson,
    start_traversal,
)
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database


class DailyPractice:
    def __init__(
        self,
        database: Database,
        clock: Callable[[], datetime] | None = None,
        rng: random.Random | None = None,
    ):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))
        self.rng = rng or random.Random()  # noqa: S311

    def open(self, language: LearningLanguage, lesson_id: str | None) -> PracticeState:
        with self.database.write() as connection:
            require_lesson(connection, lesson_id)
            now = self.clock()
            day = learning_day(connection, now)
            view = load_view(connection, language)
            if view is None or view.learning_date != day:
                # A new learning date closes an unfinished earlier view without a grade.
                start_traversal(connection, day, language, "daily", lesson_id)
            elif view.practice or not view.phrase_id:
                start_traversal(connection, day, language, view.origin, lesson_id)
            else:
                connection.execute(
                    "UPDATE current_view SET lesson_id=? WHERE language=?",
                    (lesson_id, language),
                )
            view = load_view(connection, language)
            if view is not None and not view.phrase_id:
                advance(connection, now, day, language, self.rng)
            return practice_state(connection, now, day, language)
