import json
import sqlite3
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import cast
from uuid import uuid4

from phrase_gym.server.contracts import StudyCard, StudyCompletion, StudyResponse
from phrase_gym.server.scheduling import learning_day, next_day_start, record_review
from phrase_gym.server.storage import Database
from phrase_gym.server.study_views import response, study_card

MAX_SESSION_CARDS = 20
MAX_NEW_CARDS = 5


class StudyService:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def current(self) -> StudyResponse:
        with self.database.write() as connection:
            instant = self.clock()
            day = learning_day(connection, instant)
            active = connection.execute(
                "SELECT * FROM attempts WHERE completed_at IS NULL AND kind!='free' "
                "AND card_id IN (SELECT id FROM cards WHERE suspended=0) "
                "ORDER BY created_at LIMIT 1",
            ).fetchone()
            if active:
                card = connection.execute(
                    "SELECT * FROM cards WHERE id=?",
                    (active["card_id"],),
                ).fetchone()
                return response(connection, card, active, self.clock().isoformat())
            count = connection.execute(
                "SELECT COUNT(*) FROM attempts WHERE kind IN ('study','review') AND learning_day=?",
                (day,),
            ).fetchone()[0]
            card = None
            kind = "study"
            if count < MAX_SESSION_CARDS:
                card = connection.execute(
                    "SELECT * FROM cards WHERE studied_at IS NOT NULL AND suspended=0 "
                    "AND due_at<=? AND (exposed_day IS NULL OR exposed_day!=?) "
                    "AND id NOT IN (SELECT card_id FROM attempts WHERE kind='review' "
                    "AND learning_day=?) "
                    "ORDER BY due_at, created_at LIMIT 1",
                    (instant.isoformat(), day, day),
                ).fetchone()
                if card:
                    kind = "review"
                elif (
                    connection.execute(
                        "SELECT COUNT(*) FROM attempts WHERE kind='study' AND learning_day=?",
                        (day,),
                    ).fetchone()[0]
                    < MAX_NEW_CARDS
                ):
                    card = connection.execute(
                        "SELECT * FROM cards WHERE studied_at IS NULL AND suspended=0 "
                        "ORDER BY created_at, rowid LIMIT 1",
                    ).fetchone()
            if card:
                attempt = self._start(connection, card, kind, day, instant)
                return response(connection, card, attempt, self.clock().isoformat())
            return response(connection, None, None, self.clock().isoformat())

    def _start(
        self,
        connection: sqlite3.Connection,
        card: sqlite3.Row,
        kind: str,
        day: str,
        instant: datetime,
    ) -> sqlite3.Row:
        cues = json.loads(card["cues_json"])
        cue = cues[card["cue_index"] % len(cues)]
        attempt_id = str(uuid4())
        support = (
            ["complete wording", "meaning/use help", "reference audio"] if kind != "review" else []
        )
        connection.execute(
            "INSERT INTO attempts (id,card_id,created_at,completed_at,answer_snapshot,"
            "explanation_snapshot,cue_snapshot,support_json,kind,learning_day) "
            "VALUES (?,?,?,NULL,?,?,?,?,?,?)",
            (
                attempt_id,
                card["id"],
                instant.isoformat(),
                card["answer"],
                card["explanation"],
                json.dumps(cue),
                json.dumps(support),
                kind,
                day,
            ),
        )
        return cast(
            sqlite3.Row,
            connection.execute("SELECT * FROM attempts WHERE id=?", (attempt_id,)).fetchone(),
        )

    def reveal(self, attempt_id: str, response_mode: str) -> StudyCard | None:
        with self.database.write() as connection:
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE id=?",
                (attempt_id,),
            ).fetchone()
            if not attempt or attempt["completed_at"] or attempt["kind"] != "review":
                return None
            connection.execute(
                "UPDATE attempts SET revealed_at=COALESCE(revealed_at,?), "
                "response_mode=COALESCE(response_mode,?) WHERE id=?",
                (self.clock().isoformat(), response_mode, attempt_id),
            )
            card = connection.execute(
                "SELECT * FROM cards WHERE id=?",
                (attempt["card_id"],),
            ).fetchone()
            updated = connection.execute(
                "SELECT * FROM attempts WHERE id=?",
                (attempt_id,),
            ).fetchone()
            return study_card(connection, card, updated)

    def free(self, card_id: str) -> StudyCard | None:
        with self.database.write() as connection:
            card = connection.execute("SELECT * FROM cards WHERE id=?", (card_id,)).fetchone()
            if not card:
                return None
            instant = self.clock()
            day = learning_day(connection, instant)
            connection.execute("UPDATE cards SET exposed_day=? WHERE id=?", (day, card_id))
            attempt = self._start(connection, card, "free", day, instant)
            return study_card(connection, card, attempt)

    def complete(
        self,
        attempt_id: str,
        operation_id: str,
        rating: str | None = None,
        response_mode: str | None = None,
    ) -> StudyCompletion | None:
        with self.database.write() as connection:
            previous = connection.execute(
                "SELECT result_json FROM operations WHERE id=?",
                (operation_id,),
            ).fetchone()
            if previous:
                result = StudyCompletion.model_validate_json(previous["result_json"])
                if result.attempt_id != attempt_id:
                    raise ValueError("Operation id belongs to another attempt")
                return result
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE id=?",
                (attempt_id,),
            ).fetchone()
            if not attempt:
                return None
            if attempt["completed_at"] or (
                rating and (attempt["kind"] != "review" or not attempt["revealed_at"])
            ):
                raise ValueError("Attempt already completed or not ready for a rating")
            if attempt["kind"] == "review" and not rating:
                return self._skip(connection, attempt, operation_id)
            instant = self.clock()
            if rating:
                record_review(connection, attempt, rating, response_mode, instant)
            elif attempt["kind"] == "study":
                due = next_day_start(connection, instant).isoformat()
                connection.execute(
                    "UPDATE cards SET studied_at=COALESCE(studied_at,?), due_at=? WHERE id=?",
                    (instant.isoformat(), due, attempt["card_id"]),
                )
            connection.execute(
                "UPDATE attempts SET completed_at=? WHERE id=?",
                (instant.isoformat(), attempt_id),
            )
            result = StudyCompletion(
                attempt_id=attempt_id,
                completed_at=instant.isoformat(),
                graded=bool(rating),
            )
            connection.execute(
                "INSERT INTO operations VALUES (?,?,?)",
                (operation_id, "review" if rating else "complete-study", result.model_dump_json()),
            )
            return result

    def _skip(
        self,
        connection: sqlite3.Connection,
        attempt: sqlite3.Row,
        operation_id: str,
    ) -> StudyCompletion:
        instant = self.clock().isoformat()
        connection.execute(
            "UPDATE attempts SET completed_at=? WHERE id=?",
            (instant, attempt["id"]),
        )
        connection.execute(
            "UPDATE cards SET exposed_day=? WHERE id=?",
            (attempt["learning_day"], attempt["card_id"]),
        )
        result = StudyCompletion(attempt_id=attempt["id"], completed_at=instant, graded=False)
        connection.execute(
            "INSERT INTO operations VALUES (?, 'skip-review', ?)",
            (operation_id, result.model_dump_json()),
        )
        return result

    def reference(self, card_id: str) -> Path | None:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT reference_path FROM cards WHERE id=?",
                (card_id,),
            ).fetchone()
        return Path(row["reference_path"]) if row else None
