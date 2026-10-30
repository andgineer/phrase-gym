import random
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.allowance import extend_targets
from phrase_gym.server.practice_contracts import ExtraReceipt, ExtraResult, OpenExtra
from phrase_gym.server.practice_selection import advance
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import (
    PracticeConflictError,
    require_lesson,
    start_traversal,
)
from phrase_gym.server.scheduling import learning_day
from phrase_gym.server.storage import Database


class ExtraPractice:
    def __init__(
        self,
        database: Database,
        clock: Callable[[], datetime] | None = None,
        rng: random.Random | None = None,
    ):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))
        self.rng = rng or random.Random()  # noqa: S311

    def open(self, request: OpenExtra) -> ExtraResult:
        payload = request.model_dump_json()
        with self.database.write() as connection:
            # An exact retry acknowledges the original press without touching the newer view.
            previous = connection.execute(
                "SELECT kind, request_json, result_json FROM operations WHERE id=?",
                (request.operation_id,),
            ).fetchone()
            if previous:
                if previous["kind"] != "extra" or previous["request_json"] != payload:
                    raise PracticeConflictError("This operation ID was used for another action")
                receipt = ExtraReceipt.model_validate_json(previous["result_json"])
                return ExtraResult(receipt=receipt, replayed=True, state=None)
            require_lesson(connection, request.lesson_id)
            now = self.clock()
            day = learning_day(connection, now)
            due_target, new_target = extend_targets(connection, day)
            start_traversal(connection, day, "extra", request.lesson_id)
            receipt = ExtraReceipt(
                operation_id=request.operation_id,
                learning_date=day,
                due_target=due_target,
                new_target=new_target,
            )
            connection.execute(
                "INSERT INTO operations (id, kind, request_json, result_json) "
                "VALUES (?, 'extra', ?, ?)",
                (request.operation_id, payload, receipt.model_dump_json()),
            )
            advance(connection, now, day, self.rng)
            return ExtraResult(
                receipt=receipt,
                replayed=False,
                state=practice_state(connection, now, day),
            )
