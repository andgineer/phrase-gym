import json
import random
import sqlite3
from datetime import datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.practice_views import clear_phrase, load_view, show_phrase
from phrase_gym.server.records import PhraseRecord, ViewRecord, fetch_all
from phrase_gym.server.scheduling import effective_due


def _lesson_filter(view: ViewRecord) -> tuple[str, tuple[str, ...]]:
    lesson = view.lesson_id
    if lesson:
        return " AND language=? AND lesson_id=?", (view.language, lesson)
    return " AND language=?", (view.language,)


def due_phrases(
    connection: sqlite3.Connection,
    view: ViewRecord,
    now: datetime,
) -> list[PhraseRecord]:
    clause, parameters = _lesson_filter(view)
    skipped = set(json.loads(view.skipped_json))
    rows = fetch_all(
        connection,
        PhraseRecord,
        "SELECT * FROM phrases WHERE studied_at IS NOT NULL AND due_at<=?"  # noqa: S608
        f"{clause} ORDER BY due_at, created_at, position, id",
        (now.isoformat(), *parameters),
    )
    return [row for row in rows if row.id not in skipped and effective_due(connection, row) <= now]


def new_phrases(connection: sqlite3.Connection, view: ViewRecord) -> list[PhraseRecord]:
    clause, parameters = _lesson_filter(view)
    skipped = set(json.loads(view.skipped_json))
    rows = fetch_all(
        connection,
        PhraseRecord,
        f"SELECT * FROM phrases WHERE studied_at IS NULL{clause} "  # noqa: S608
        "ORDER BY created_at, position, id",
        parameters,
    )
    return [row for row in rows if row.id not in skipped]


def draw_phrase(
    connection: sqlite3.Connection,
    view: ViewRecord,
    now: datetime,
    day: str,
    rng: random.Random,
) -> tuple[PhraseRecord, str] | None:
    progress = daily_progress(connection, day, view.language)
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
    language: LearningLanguage,
    rng: random.Random,
) -> None:
    view = load_view(connection, language)
    if view is None:
        return
    drawn = draw_phrase(connection, view, now, day, rng)
    if drawn is None:
        clear_phrase(connection, language)
        return
    phrase, kind = drawn
    show_phrase(connection, phrase, kind, rng)
