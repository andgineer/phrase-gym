import json
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path

from phrase_gym.server.contracts import (
    PhraseDetails,
    PhrasePage,
    PhraseSummary,
    RecordingPage,
)
from phrase_gym.server.scheduling import effective_due, learning_day
from phrase_gym.server.search_terms import query_terms, word_prefix_filter
from phrase_gym.server.storage import Database
from phrase_gym.server.study_views import saved_recording


class LibraryService:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def phrases(self, query: str, lesson_id: str | None, offset: int, limit: int) -> PhrasePage:
        terms = query_terms(query)
        clause, parameters = word_prefix_filter("phrases.search_text", terms)
        primary, primary_parameters = word_prefix_filter("phrases.search_primary", terms)
        if lesson_id:
            clause += " AND phrases.lesson_id=?"
            parameters.append(lesson_id)
        with self.database.read() as connection:
            rows = connection.execute(
                "SELECT phrases.id, phrases.text, phrases.contexts_json, "  # noqa: S608
                f"lessons.title AS lesson_title, CASE WHEN {primary} THEN 0 ELSE 1 END AS rank "
                "FROM phrases JOIN lessons ON lessons.id=phrases.lesson_id "
                f"WHERE {clause} ORDER BY rank, phrases.created_at, phrases.position, phrases.id "
                "LIMIT ? OFFSET ?",
                (*primary_parameters, *parameters, limit + 1, offset),
            ).fetchall()
        return PhrasePage(
            items=[
                PhraseSummary(
                    id=row["id"],
                    text=row["text"],
                    usage_contexts=json.loads(row["contexts_json"]),
                    lesson_title=row["lesson_title"],
                )
                for row in rows[:limit]
            ],
            next_offset=offset + limit if len(rows) > limit else None,
        )

    def phrase(self, phrase_id: str) -> PhraseDetails | None:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT phrases.*, lessons.title AS lesson_title FROM phrases "
                "JOIN lessons ON lessons.id=phrases.lesson_id WHERE phrases.id=?",
                (phrase_id,),
            ).fetchone()
            if row is None:
                return None
            due = (
                learning_day(connection, effective_due(connection, row))
                if row["studied_at"]
                else None
            )
        return PhraseDetails(
            id=row["id"],
            text=row["text"],
            explanation=row["explanation"],
            detailed_explanation=row["detailed_explanation"],
            usage_contexts=json.loads(row["contexts_json"]),
            accepted_variants=json.loads(row["variants_json"]),
            lesson_id=row["lesson_id"],
            lesson_title=row["lesson_title"],
            source_label=row["source_label"],
            source_link=row["source_link"],
            reference_url=f"/api/phrases/{row['id']}/reference",
            studied=bool(row["studied_at"]),
            due_date=due,
        )

    def recordings(self, phrase_id: str, offset: int, limit: int) -> RecordingPage | None:
        with self.database.read() as connection:
            if not connection.execute("SELECT 1 FROM phrases WHERE id=?", (phrase_id,)).fetchone():
                return None
            rows = connection.execute(
                "SELECT * FROM recordings WHERE phrase_id=? "
                "ORDER BY captured_at DESC, id DESC LIMIT ? OFFSET ?",
                (phrase_id, limit + 1, offset),
            ).fetchall()
        return RecordingPage(
            items=[saved_recording(row) for row in rows[:limit]],
            next_offset=offset + limit if len(rows) > limit else None,
        )

    def reference(self, phrase_id: str) -> Path | None:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT reference_path FROM phrases WHERE id=?",
                (phrase_id,),
            ).fetchone()
        return Path(row["reference_path"]) if row else None
