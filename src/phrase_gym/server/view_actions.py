import json
import random
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.import_contracts import Cue
from phrase_gym.server.practice_contracts import PracticeCard, ViewAction, ViewOperation
from phrase_gym.server.practice_views import (
    PracticeConflictError,
    choose,
    load_view,
    practice_card,
    require_view,
    view_phrase,
)
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database

# Enough retained choices for a client's exact retries; older operation IDs are forgotten.
MAX_VIEW_OPERATIONS = 8
CHOICES = {"cue": ("cue_index", "cues_json"), "hint": ("hint_index", "hints_json")}


class ViewActions:
    def __init__(
        self,
        database: Database,
        clock: Callable[[], datetime] | None = None,
        rng: random.Random | None = None,
    ):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))
        self.rng = rng or random.Random()  # noqa: S311

    def reveal(self, action: ViewAction) -> PracticeCard:
        with self.database.write() as connection:
            view = require_view(connection, learning_day(connection, self.clock()), action)
            connection.execute("UPDATE current_view SET revealed=1 WHERE id=1")
            view = load_view(connection) or view
            return practice_card(connection, view, view_phrase(connection, view))

    def new_cue(self, operation: ViewOperation) -> PracticeCard:
        return self._choose(operation, "cue")

    def hint(self, operation: ViewOperation) -> PracticeCard:
        return self._choose(operation, "hint")

    def _choose(self, operation: ViewOperation, kind: str) -> PracticeCard:
        column, values_column = CHOICES[kind]
        with self.database.write() as connection:
            view = require_view(connection, learning_day(connection, self.clock()), operation)
            if view["kind"] != "recall":
                raise PracticeConflictError("Cues and hints are offered only for recall")
            phrase = view_phrase(connection, view)
            values = json.loads(phrase[values_column])
            history = json.loads(view["operations_json"])
            prior = history.get(operation.operation_id)
            if prior is not None:
                if prior["kind"] != kind:
                    raise PracticeConflictError("This operation ID was used for another action")
                card = practice_card(connection, view, phrase)
                chosen = values[prior["choice"]]
                update = {"cue": Cue(**chosen)} if kind == "cue" else {"hint": chosen}
                return card.model_copy(update=update)
            choice = choose(self.rng, len(values), view[column])
            history[operation.operation_id] = {"kind": kind, "choice": choice}
            retained = dict(list(history.items())[-MAX_VIEW_OPERATIONS:])
            connection.execute(
                f"UPDATE current_view SET {column}=?, operations_json=? WHERE id=1",  # noqa: S608
                (choice, json.dumps(retained)),
            )
            if kind == "cue":
                connection.execute(
                    "UPDATE phrases SET last_cue=? WHERE id=?",
                    (choice, phrase["id"]),
                )
            view = load_view(connection) or view
            return practice_card(connection, view, phrase)
