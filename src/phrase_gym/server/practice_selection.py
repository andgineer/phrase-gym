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


def new_phrases(connection: sqlite3.Connection, view: sqlite3.Row) -> list[sqlite3.Row]:
    clause, parameters = _lesson_filter(view)
    skipped = set(json.loads(view["skipped_json"]))
    rows = connection.execute(
        f"SELECT * FROM phrases WHERE studied_at IS NULL{clause} "  # noqa: S608
        "ORDER BY created_at, position, id",
        parameters,
    ).fetchall()
    return [row for row in rows if row["id"] not in skipped]


def draw_phrase(
    connection: sqlite3.Connection,
    view: sqlite3.Row,
    now: datetime,
    day: str,
    rng: random.Random,
) -> tuple[sqlite3.Row, str] | None:
    progress = daily_progress(connection, day)
    due_left = max(progress.due_allowance - progress.due_completed, 0)
    new_left = max(progress.new_allowance - progress.new_completed, 0)
    # FSRS needs the longest-due phrases reviewed first; only their order within the session varies.
    due = due_phrases(connection, view, now)[:due_left]
    fresh = new_phrases(connection, view)
    new_slots = min(new_left, len(fresh))
    if not due and not new_slots:
        return None
    # Weighting by remaining slots shuffles due and new phrases uniformly through the session.
    if rng.randrange(len(due) + new_slots) < len(due):
        return rng.choice(due), "recall"
    return rng.choice(fresh), "new"


def advance(
    connection: sqlite3.Connection,
    now: datetime,
    day: str,
    rng: random.Random,
) -> None:
    view = load_view(connection)
    if view is None:
        return
    drawn = draw_phrase(connection, view, now, day, rng)
    if drawn is None:
        clear_phrase(connection)
        return
    phrase, kind = drawn
    show_phrase(connection, phrase, kind, rng)
