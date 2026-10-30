import random
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.practice_contracts import PracticeState
from phrase_gym.server.practice_selection import advance
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import REGULAR_ORIGINS, load_view, start_traversal
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database


class UnknownLessonError(KeyError):
    pass


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

    def open(self, lesson_id: str | None) -> PracticeState:
        with self.database.write() as connection:
            if (
                lesson_id
                and not connection.execute(
                    "SELECT 1 FROM lessons WHERE id=?",
                    (lesson_id,),
                ).fetchone()
            ):
                raise UnknownLessonError(lesson_id)
            now = self.clock()
            day = learning_day(connection, now)
            view = load_view(connection)
            if (
                view is None
                or view["learning_date"] != day
                or view["origin"] not in REGULAR_ORIGINS
            ):
                # A new learning date closes an unfinished earlier view without a grade.
                start_traversal(connection, day, "daily", lesson_id)
            else:
                connection.execute("UPDATE current_view SET lesson_id=? WHERE id=1", (lesson_id,))
            view = load_view(connection)
            if view is not None and not view["phrase_id"]:
                advance(connection, now, day, self.rng)
            return practice_state(connection, now, day)
