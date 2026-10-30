import json
import random
import sqlite3
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.practice_contracts import (
    CompletionReceipt,
    CompletionRequest,
    CompletionResult,
    Outcome,
    Rating,
)
from phrase_gym.server.practice_selection import advance
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import PracticeConflictError, require_view, view_phrase
from phrase_gym.server.scheduling import graded_today, learning_day, next_day_start, schedule
from phrase_gym.server.storage import Database


def _record_outcome(
    connection: sqlite3.Connection,
    view: sqlite3.Row,
    phrase: sqlite3.Row,
    outcome: Outcome,
    now: datetime,
) -> tuple[Rating | None, bool, bool]:
    if view["kind"] == "first_study":
        if outcome != "studied":
            raise PracticeConflictError("A first meeting completes with Next, without a rating")
        # First study schedules tomorrow's recall but is not a review.
        connection.execute(
            "UPDATE phrases SET studied_at=COALESCE(studied_at, ?), due_at=? WHERE id=?",
            (now.isoformat(), next_day_start(connection, now).isoformat(), phrase["id"]),
        )
        return None, False, False
    if outcome == "studied":
        raise PracticeConflictError("Recall completes with a rating or Skip")
    if outcome == "skipped":
        skipped = json.loads(view["skipped_json"])
        connection.execute(
            "UPDATE current_view SET skipped_json=? WHERE id=1",
            (json.dumps([*skipped, phrase["id"]]),),
        )
        return None, True, False
    if not view["revealed"]:
        raise PracticeConflictError("Show the phrase before rating your answer")
    if graded_today(connection, phrase, learning_day(connection, now)):
        return outcome, False, False
    fsrs_json, due = schedule(connection, phrase["fsrs_json"], outcome, now)
    connection.execute(
        "UPDATE phrases SET fsrs_json=?, due_at=?, last_review=? WHERE id=?",
        (fsrs_json, due, now.isoformat(), phrase["id"]),
    )
    return outcome, False, True


class PracticeCompletion:
    def __init__(
        self,
        database: Database,
        clock: Callable[[], datetime] | None = None,
        rng: random.Random | None = None,
    ):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))
        self.rng = rng or random.Random()  # noqa: S311

    def complete(self, request: CompletionRequest) -> CompletionResult:
        payload = request.model_dump_json()
        with self.database.write() as connection:
            # A persisted completion answers its exact retry before the view is checked.
            previous = connection.execute(
                "SELECT kind, request_json, result_json FROM operations WHERE id=?",
                (request.completion_id,),
            ).fetchone()
            if previous:
                if previous["kind"] != "completion" or previous["request_json"] != payload:
                    raise PracticeConflictError("This completion ID was used for another action")
                receipt = CompletionReceipt.model_validate_json(previous["result_json"])
                return CompletionResult(receipt=receipt, replayed=True, state=None)
            now = self.clock()
            day = learning_day(connection, now)
            view = require_view(connection, day, request)
            phrase = view_phrase(connection, view)
            rating, skipped, fsrs_updated = _record_outcome(
                connection,
                view,
                phrase,
                request.outcome,
                now,
            )
            receipt = CompletionReceipt(
                completion_id=request.completion_id,
                phrase_id=phrase["id"],
                learning_date=day,
                mode=view["origin"],
                kind=view["kind"],
                rating=rating,
                skipped=skipped,
                fsrs_updated=fsrs_updated,
            )
            connection.execute(
                "INSERT INTO practice_log (id, phrase_id, phrase_text, lesson_title, "
                "completed_at, learning_date, kind, rating, skipped, fsrs_updated, mode) "
                "VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                (
                    receipt.completion_id,
                    phrase["id"],
                    phrase["text"],
                    phrase["lesson_title"],
                    now.isoformat(),
                    day,
                    receipt.kind,
                    rating,
                    int(skipped),
                    int(fsrs_updated),
                    receipt.mode,
                ),
            )
            connection.execute(
                "INSERT INTO operations (id, kind, request_json, result_json) "
                "VALUES (?, 'completion', ?, ?)",
                (request.completion_id, payload, receipt.model_dump_json()),
            )
            advance(connection, now, day, self.rng)
            return CompletionResult(
                receipt=receipt,
                replayed=False,
                state=practice_state(connection, now, day),
            )
