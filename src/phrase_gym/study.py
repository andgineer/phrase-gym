import json
import sqlite3
from datetime import UTC, datetime
from pathlib import Path
from typing import cast
from uuid import uuid4

from phrase_gym.contracts import (
    SavedRecording,
    StudyCard,
    StudyCompletion,
    StudyHistoryItem,
    StudyResponse,
)
from phrase_gym.storage import Database


def now() -> str:
    return datetime.now(UTC).isoformat()


class StudyService:
    def __init__(self, database: Database):
        self.database = database

    @staticmethod
    def recordings(connection: sqlite3.Connection, card_id: str) -> list[SavedRecording]:
        rows = connection.execute(
            "SELECT id, attempt_id, captured_at, mime_type, duration_ms, complete, byte_length "
            "FROM recordings WHERE card_id=? ORDER BY captured_at, id",
            (card_id,),
        ).fetchall()
        return [
            SavedRecording(
                id=row["id"],
                attempt_id=row["attempt_id"],
                captured_at=row["captured_at"],
                mime_type=row["mime_type"],
                duration_ms=row["duration_ms"],
                complete=bool(row["complete"]),
                byte_length=row["byte_length"],
                url=f"/api/recordings/{row['id']}/audio",
            )
            for row in rows
        ]

    def current(self) -> StudyResponse:
        with self.database.write() as connection:
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE completed_at IS NULL ORDER BY created_at LIMIT 1"
            ).fetchone()
            card = self._attempt_card(connection, attempt) if attempt else None
            if not card:
                card = connection.execute(
                    "SELECT * FROM cards WHERE studied_at IS NULL "
                    "ORDER BY created_at, rowid LIMIT 1"
                ).fetchone()
                if not card:
                    return self._complete_state(connection)
                attempt = self._start_attempt(connection, card)
            assert attempt is not None
            total, studied = self._counts(connection)
            return StudyResponse(
                done=False,
                total=total,
                studied=studied,
                card=self._card(connection, card, attempt["id"]),
            )

    def history(self) -> list[StudyHistoryItem]:
        with self.database.read() as connection:
            cards = connection.execute(
                "SELECT * FROM cards WHERE studied_at IS NOT NULL ORDER BY studied_at DESC"
            ).fetchall()
            return [
                StudyHistoryItem(
                    id=card["id"],
                    answer=card["answer"],
                    source_label=card["source_label"],
                    reference_url=f"/api/cards/{card['id']}/reference",
                    recordings=self.recordings(connection, card["id"]),
                )
                for card in cards
            ]

    def complete(self, attempt_id: str, operation_id: str) -> StudyCompletion | None:
        with self.database.write() as connection:
            previous = connection.execute(
                "SELECT result_json FROM operations WHERE id=?", (operation_id,)
            ).fetchone()
            if previous:
                return StudyCompletion.model_validate_json(previous["result_json"])
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE id=?", (attempt_id,)
            ).fetchone()
            if not attempt:
                return None
            completed_at = attempt["completed_at"] or now()
            connection.execute(
                "UPDATE attempts SET completed_at=? WHERE id=?", (completed_at, attempt_id)
            )
            connection.execute(
                "UPDATE cards SET studied_at=COALESCE(studied_at, ?) WHERE id=?",
                (completed_at, attempt["card_id"]),
            )
            result = StudyCompletion(attempt_id=attempt_id, completed_at=completed_at, graded=False)
            connection.execute(
                "INSERT INTO operations VALUES (?, 'complete-study', ?)",
                (operation_id, result.model_dump_json()),
            )
            return result

    def reference(self, card_id: str) -> Path | None:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT reference_path FROM cards WHERE id=?", (card_id,)
            ).fetchone()
        return Path(row["reference_path"]) if row else None

    @staticmethod
    def _attempt_card(connection: sqlite3.Connection, attempt: sqlite3.Row) -> sqlite3.Row | None:
        return cast(
            sqlite3.Row | None,
            connection.execute("SELECT * FROM cards WHERE id=?", (attempt["card_id"],)).fetchone(),
        )

    @staticmethod
    def _start_attempt(connection: sqlite3.Connection, card: sqlite3.Row) -> sqlite3.Row:
        attempt_id = str(uuid4())
        cues = json.loads(card["cues_json"])
        connection.execute(
            "INSERT INTO attempts VALUES (?, ?, ?, NULL, ?, ?, ?, ?)",
            (
                attempt_id,
                card["id"],
                now(),
                card["answer"],
                card["explanation"],
                json.dumps(cues[0]),
                json.dumps(["complete wording", "meaning/use help", "reference audio"]),
            ),
        )
        attempt = connection.execute("SELECT * FROM attempts WHERE id=?", (attempt_id,)).fetchone()
        assert attempt is not None
        return cast(sqlite3.Row, attempt)

    @staticmethod
    def _counts(connection: sqlite3.Connection) -> tuple[int, int]:
        total = connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
        studied = connection.execute(
            "SELECT COUNT(*) FROM cards WHERE studied_at IS NOT NULL"
        ).fetchone()[0]
        return total, studied

    def _complete_state(self, connection: sqlite3.Connection) -> StudyResponse:
        total, studied = self._counts(connection)
        return StudyResponse(
            done=True,
            total=total,
            studied=studied,
            message=("Study complete. Your phrases are saved; later recall is not scheduled yet."),
        )

    def _card(self, connection: sqlite3.Connection, row: sqlite3.Row, attempt_id: str) -> StudyCard:
        return StudyCard(
            id=row["id"],
            source_id=row["source_id"],
            item_id=row["item_id"],
            source_label=row["source_label"],
            source_link=row["source_link"],
            answer=row["answer"],
            explanation=row["explanation"],
            cues=json.loads(row["cues_json"]),
            accepted_variants=json.loads(row["variants_json"]),
            detailed_explanation=row["detailed_explanation"],
            reference_url=f"/api/cards/{row['id']}/reference",
            attempt_id=attempt_id,
            recordings=self.recordings(connection, row["id"]),
        )
