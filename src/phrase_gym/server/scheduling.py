import json
import sqlite3
from datetime import UTC, datetime, timedelta
from uuid import uuid4
from zoneinfo import ZoneInfo

from fsrs import Card, Rating, Scheduler

RATINGS = {"not_yet": Rating.Again, "with_effort": Rating.Hard, "recalled": Rating.Good}
SCHEDULER = Scheduler(learning_steps=(), relearning_steps=(), enable_fuzzing=False)


def learning_day(connection: sqlite3.Connection, instant: datetime) -> str:
    zone = connection.execute("SELECT learning_timezone FROM settings WHERE id=1").fetchone()[0]
    return instant.astimezone(ZoneInfo(zone)).date().isoformat()


def next_day_start(connection: sqlite3.Connection, instant: datetime) -> datetime:
    setting = connection.execute("SELECT learning_timezone FROM settings WHERE id=1").fetchone()[0]
    zone = ZoneInfo(setting)
    tomorrow = instant.astimezone(zone).date() + timedelta(days=1)
    return datetime.combine(tomorrow, datetime.min.time(), zone).astimezone(UTC)


def schedule(
    fsrs_json: str | None,
    rating: str,
    instant: datetime,
    connection: sqlite3.Connection,
) -> tuple[str, str]:
    card = Card.from_dict(json.loads(fsrs_json)) if fsrs_json else Card()
    updated, _ = SCHEDULER.review_card(card, RATINGS[rating], review_datetime=instant)
    due = max(updated.due, next_day_start(connection, instant))
    updated.due = due
    return json.dumps(updated.to_dict()), due.isoformat()


def record_review(
    connection: sqlite3.Connection,
    attempt: sqlite3.Row,
    rating: str,
    response_mode: str | None,
    instant: datetime,
) -> None:
    day = learning_day(connection, instant)
    if connection.execute(
        "SELECT 1 FROM reviews WHERE card_id=? AND learning_day=?",
        (attempt["card_id"], day),
    ).fetchone():
        raise ValueError("Independent review is limited to one per learning day")
    card = connection.execute("SELECT * FROM cards WHERE id=?", (attempt["card_id"],)).fetchone()
    if card["suspended"]:
        raise ValueError("Suspended cards cannot be rated")
    if card["exposed_day"] == day:
        raise ValueError("This card was viewed today; choose Study / skip grading")
    fsrs_json, due = schedule(card["fsrs_json"], rating, instant, connection)
    connection.execute(
        "UPDATE cards SET fsrs_json=?, due_at=?, cue_index=cue_index+1 WHERE id=?",
        (fsrs_json, due, card["id"]),
    )
    connection.execute(
        "INSERT INTO reviews VALUES (?,?,?,?,?,?,?,?,?,?)",
        (
            str(uuid4()),
            attempt["id"],
            card["id"],
            instant.isoformat(),
            day,
            attempt["cue_snapshot"],
            attempt["response_mode"] or response_mode or "silent",
            0,
            rating,
            due,
        ),
    )
