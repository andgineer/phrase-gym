import json
import random
import sqlite3
from datetime import datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.practice_views import clear_phrase, load_view, show_phrase
from phrase_gym.server.scheduling import effective_due


def _lesson_filter(view: sqlite3.Row) -> tuple[str, tuple[str, ...]]:
    lesson = view["lesson_id"]
    return (" AND lesson_id=?", (lesson,)) if lesson else ("", ())


def due_phrases(
    connection: sqlite3.Connection,
    view: sqlite3.Row,
    now: datetime,
) -> list[sqlite3.Row]:
    clause, parameters = _lesson_filter(view)
    skipped = set(json.loads(view["skipped_json"]))
    rows = connection.execute(
        "SELECT * FROM phrases WHERE studied_at IS NOT NULL AND due_at<=?"  # noqa: S608
        f"{clause} ORDER BY due_at, created_at, position, id",
        (now.isoformat(), *parameters),
    ).fetchall()
    return [
        row for row in rows if row["id"] not in skipped and effective_due(connection, row) <= now
    ]


def first_new_phrase(connection: sqlite3.Connection, view: sqlite3.Row) -> sqlite3.Row | None:
    clause, parameters = _lesson_filter(view)
    return connection.execute(
        f"SELECT * FROM phrases WHERE studied_at IS NULL{clause} "  # noqa: S608
        "ORDER BY created_at, position, id LIMIT 1",
        parameters,
    ).fetchone()


def advance(
    connection: sqlite3.Connection,
    now: datetime,
    day: str,
    rng: random.Random,
) -> None:
    view = load_view(connection)
    if view is None:
        return
    progress = daily_progress(connection, day)
    if progress.due_completed < progress.due_allowance:
        due = due_phrases(connection, view, now)
        if due:
            show_phrase(connection, due[0], "recall", rng)
            return
    if progress.new_completed < progress.new_allowance:
        fresh = first_new_phrase(connection, view)
        if fresh:
            show_phrase(connection, fresh, "first_study", rng)
            return
    clear_phrase(connection)
