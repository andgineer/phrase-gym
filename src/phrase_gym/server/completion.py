import json
import random
import sqlite3
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.practice_contracts import (
    CompletionReceipt,
    CompletionRequest,
    CompletionResult,
    GradeSkippedRequest,
    Outcome,
    PracticeState,
    Rating,
)
from phrase_gym.server.practice_selection import advance
from phrase_gym.server.practice_state import practice_state
from phrase_gym.server.practice_views import (
    PracticeConflictError,
    StaleViewError,
    clear_phrase,
    require_view,
    view_mode,
    view_phrase,
)
from phrase_gym.server.records import PhraseRecord, ShownPhraseRecord, ViewRecord, fetch_one
from phrase_gym.server.scheduling import graded_today, learning_day, schedule
from phrase_gym.server.storage import Database


def _record_outcome(
    connection: sqlite3.Connection,
    view: ViewRecord,
    phrase: ShownPhraseRecord,
    outcome: Outcome,
    now: datetime,
) -> tuple[Rating | None, bool, bool]:
    if outcome == "studied":
        raise PracticeConflictError("Practice completes with a rating or Skip")
    if outcome == "skipped":
        if view.practice:
            return None, True, False
        skipped = json.loads(view.skipped_json)
        connection.execute(
            "UPDATE current_view SET skipped_json=? WHERE id=1",
            (json.dumps([*skipped, phrase.id]),),
        )
        return None, True, False
    if not view.revealed:
        raise PracticeConflictError("Show the phrase before rating your answer")
    return outcome, False, _apply_rating(connection, phrase, outcome, now)


def _apply_rating(
    connection: sqlite3.Connection,
    phrase: PhraseRecord,
    rating: Rating,
    now: datetime,
) -> bool:
    if graded_today(connection, phrase, learning_day(connection, now)):
        return False
    fsrs_json, due = schedule(connection, phrase.fsrs_json, rating, now)
    connection.execute(
        "UPDATE phrases SET fsrs_json=?, due_at=?, last_review=?, "
        "studied_at=COALESCE(studied_at, ?) WHERE id=?",
        (fsrs_json, due, now.isoformat(), now.isoformat(), phrase.id),
    )
    return True


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
                if (
                    previous["kind"] not in {"completion", "revealed_skip"}
                    or previous["request_json"] != payload
                ):
                    raise PracticeConflictError("This completion ID was used for another action")
                receipt = CompletionReceipt.model_validate_json(previous["result_json"])
                return CompletionResult(receipt=receipt, replayed=True, state=None)
            now = self.clock()
            day = learning_day(connection, now)
            view = require_view(connection, day, request)
            phrase = view_phrase(connection, view)
            if view.kind is None:
                raise StaleViewError("This card is no longer current. Open Daily to continue.")
            rating, skipped, fsrs_updated = _record_outcome(
                connection,
                view,
                phrase,
                request.outcome,
                now,
            )
            receipt = CompletionReceipt(
                completion_id=request.completion_id,
                phrase_id=phrase.id,
                learning_date=day,
                mode=view_mode(view),
                kind=view.kind,
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
                    phrase.id,
                    phrase.text,
                    phrase.lesson_title,
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
                "INSERT INTO operations (id, kind, request_json, result_json) VALUES (?, ?, ?, ?)",
                (
                    request.completion_id,
                    "revealed_skip"
                    if skipped and view.revealed and not view.practice
                    else "completion",
                    payload,
                    receipt.model_dump_json(),
                ),
            )
            if view.practice:
                # Practice returns to details; the next Daily selects a fresh regular view.
                clear_phrase(connection)
                state = PracticeState(
                    card=None,
                    progress=daily_progress(connection, day),
                    message=None,
                )
            else:
                advance(connection, now, day, self.rng)
                state = practice_state(connection, now, day)
            return CompletionResult(receipt=receipt, replayed=False, state=state)

    def grade_skipped(self, request: GradeSkippedRequest) -> CompletionReceipt:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT practice_log.phrase_id, practice_log.skipped, practice_log.rating, "
                "practice_log.fsrs_updated, operations.result_json "
                "FROM practice_log JOIN operations ON operations.id=practice_log.id "
                "WHERE practice_log.id=? AND operations.kind='revealed_skip'",
                (request.completion_id,),
            ).fetchone()
            if row is None:
                raise PracticeConflictError("This skipped card cannot be graded.")
            receipt = CompletionReceipt.model_validate_json(row["result_json"])
            if not row["skipped"]:
                if row["rating"] != request.rating:
                    raise PracticeConflictError("This card already has a saved grade.")
                updated = bool(row["fsrs_updated"])
            else:
                now = self.clock()
                phrase = fetch_one(
                    connection,
                    PhraseRecord,
                    "SELECT * FROM phrases WHERE id=?",
                    (row["phrase_id"],),
                )
                if phrase is None or receipt.learning_date != learning_day(connection, now):
                    raise PracticeConflictError(
                        "This skipped card is no longer available to grade.",
                    )
                updated = _apply_rating(connection, phrase, request.rating, now)
                # Replace the skip in place so the appearance consumes only one slot.
                connection.execute(
                    "UPDATE practice_log SET rating=?, skipped=0, fsrs_updated=? WHERE id=?",
                    (request.rating, int(updated), request.completion_id),
                )
            return receipt.model_copy(
                update={"rating": request.rating, "skipped": False, "fsrs_updated": updated},
            )
