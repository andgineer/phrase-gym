import json
import sqlite3
from datetime import UTC, datetime, timedelta
from zoneinfo import ZoneInfo

from fsrs import Card, Rating, Scheduler

RATINGS = {"not_yet": Rating.Again, "effort": Rating.Hard, "got_it": Rating.Good}
SCHEDULER = Scheduler(learning_steps=(), relearning_steps=(), enable_fuzzing=False)


def learning_zone(connection: sqlite3.Connection) -> ZoneInfo:
    setting = connection.execute("SELECT learning_timezone FROM settings WHERE id=1").fetchone()
    return ZoneInfo(setting[0])


def learning_day(connection: sqlite3.Connection, instant: datetime) -> str:
    return instant.astimezone(learning_zone(connection)).date().isoformat()


def next_day_start(connection: sqlite3.Connection, instant: datetime) -> datetime:
    zone = learning_zone(connection)
    tomorrow = instant.astimezone(zone).date() + timedelta(days=1)
    return datetime.combine(tomorrow, datetime.min.time(), zone).astimezone(UTC)


def effective_due(connection: sqlite3.Connection, phrase: sqlite3.Row) -> datetime:
    # A changed timezone must not reopen a phrase on the learning day it was studied or graded.
    latest = phrase["last_review"] or phrase["studied_at"]
    due = datetime.fromisoformat(phrase["due_at"])
    return max(due, next_day_start(connection, datetime.fromisoformat(latest)))


def graded_today(connection: sqlite3.Connection, phrase: sqlite3.Row, day: str) -> bool:
    last = phrase["last_review"]
    return last is not None and learning_day(connection, datetime.fromisoformat(last)) == day


def schedule(
    connection: sqlite3.Connection,
    fsrs_json: str | None,
    rating: str,
    instant: datetime,
) -> tuple[str, str]:
    card = Card.from_dict(json.loads(fsrs_json)) if fsrs_json else Card()
    updated, _ = SCHEDULER.review_card(card, RATINGS[rating], review_datetime=instant)
    updated.due = max(updated.due, next_day_start(connection, instant))
    return json.dumps(updated.to_dict()), updated.due.isoformat()
