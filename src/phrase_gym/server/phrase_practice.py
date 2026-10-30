import random
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.practice_contracts import PracticeState
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import load_view, show_phrase, start_traversal
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database


class UnknownPhraseError(KeyError):
    pass


class PhrasePractice:
    def __init__(
        self,
        database: Database,
        clock: Callable[[], datetime] | None = None,
        rng: random.Random | None = None,
    ):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))
        self.rng = rng or random.Random()  # noqa: S311

    def open(self, phrase_id: str) -> PracticeState:
        with self.database.write() as connection:
            phrase = connection.execute(
                "SELECT * FROM phrases WHERE id=?",
                (phrase_id,),
            ).fetchone()
            if phrase is None:
                raise UnknownPhraseError(phrase_id)
            now = self.clock()
            day = learning_day(connection, now)
            view = load_view(connection)
            if view is None or view["learning_date"] != day:
                start_traversal(connection, day, "daily", None)
            # Replacing the active phrase closes any unfinished regular view ungraded while its
            # traversal keeps today's skipped phrases.
            kind = "recall" if phrase["studied_at"] else "first_study"
            show_phrase(connection, phrase, kind, self.rng, practice=True)
            return practice_state(connection, now, day)
