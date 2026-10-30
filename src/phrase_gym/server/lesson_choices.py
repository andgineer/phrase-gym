import sqlite3
from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.contracts import LessonChoice, LessonPage
from phrase_gym.server.phrase_counts import schedule_summary
from phrase_gym.server.search_terms import query_terms, word_prefix_filter
from phrase_gym.server.storage import Database


class LessonChoices:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def page(self, query: str, offset: int, limit: int) -> LessonPage:
        clause, parameters = word_prefix_filter("search_text", query_terms(query))
        now = self.clock()
        with self.database.read() as connection:
            lessons = connection.execute(
                f"SELECT * FROM lessons WHERE {clause} "  # noqa: S608
                "ORDER BY created_at, position, title, id LIMIT ? OFFSET ?",
                (*parameters, limit + 1, offset),
            ).fetchall()
            schedule = "SELECT lesson_id, studied_at, due_at, fsrs_json FROM phrases"
            phrases = connection.execute(schedule).fetchall()
            by_lesson: dict[str, list[sqlite3.Row]] = {}
            for row in phrases:
                by_lesson.setdefault(row["lesson_id"], []).append(row)
            items = []
            for lesson in lessons[:limit]:
                counts, _ = schedule_summary(connection, by_lesson.get(lesson["id"], []), now)
                items.append(
                    LessonChoice(
                        id=lesson["id"],
                        title=lesson["title"],
                        source_label=lesson["source_label"],
                        **counts.model_dump(),
                    ),
                )
            all_lessons, _ = schedule_summary(connection, phrases, now)
        return LessonPage(
            items=items,
            next_offset=offset + limit if len(lessons) > limit else None,
            all_lessons=all_lessons,
        )
