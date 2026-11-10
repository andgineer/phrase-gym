from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.practice_contracts import PracticeState
from phrase_gym.server.practice_views import (
    StaleViewError,
    load_view,
    practice_card,
    view_phrase,
)
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database


class CurrentPractice:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def read(self) -> PracticeState:
        # Reconciliation only reads, so it never closes another tab's Practice or an old view.
        now = self.clock()
        with self.database.snapshot() as connection:
            day = learning_day(connection, now)
            view = load_view(connection)
            phrase = None
            if (
                view is not None
                and view.learning_date == day
                and view.phrase_id
                and not view.practice
            ):
                try:
                    phrase = view_phrase(connection, view)
                except StaleViewError:
                    phrase = None
            card = practice_card(connection, view, phrase, now) if view and phrase else None
            return PracticeState(card=card, progress=daily_progress(connection, day), message=None)
