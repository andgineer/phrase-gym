import sqlite3
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.contracts import LessonChoice, LessonPage
from phrase_gym.server.phrase_counts import SCHEDULE_COLUMNS, schedule_summary
from phrase_gym.server.search_terms import query_terms, word_prefix_filter
from phrase_gym.server.storage import Database


class LessonChoices:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def page(self, query: str, lesson_id: str | None, offset: int, limit: int) -> LessonPage:
        clause, parameters = word_prefix_filter("search_text", query_terms(query))
        now = self.clock()
        with self.database.read() as connection:
            lessons = connection.execute(
                f"SELECT * FROM lessons WHERE {clause} "  # noqa: S608
                "ORDER BY created_at, position, title, id LIMIT ? OFFSET ?",
                (*parameters, limit + 1, offset),
            ).fetchall()
            selected = (
                connection.execute("SELECT * FROM lessons WHERE id=?", (lesson_id,)).fetchone()
                if lesson_id
                else None
            )
            phrases = connection.execute(
                f"SELECT lesson_id, {SCHEDULE_COLUMNS} FROM phrases",  # noqa: S608
            ).fetchall()
            by_lesson: dict[str, list[sqlite3.Row]] = {}
            for row in phrases:
                by_lesson.setdefault(row["lesson_id"], []).append(row)

            def choice(lesson: sqlite3.Row) -> LessonChoice:
                counts = schedule_summary(connection, by_lesson.get(lesson["id"], []), now)
                return LessonChoice(
                    id=lesson["id"],
                    title=lesson["title"],
                    source_label=lesson["source_label"],
                    **counts.model_dump(),
                )

            return LessonPage(
                items=[choice(lesson) for lesson in lessons[:limit]],
                next_offset=offset + limit if len(lessons) > limit else None,
                all_lessons=schedule_summary(connection, phrases, now),
                selected=choice(selected) if selected else None,
            )
