import json
import sqlite3
from datetime import UTC, date, datetime, timedelta
from zoneinfo import ZoneInfo

from fsrs import Card, Rating, Scheduler

from phrase_gym.server.contracts import ReturnPace
from phrase_gym.server.settings_store import read_settings

RATINGS = {
    "not_yet": Rating.Again,
    "effort": Rating.Hard,
    "got_it": Rating.Good,
    "easy": Rating.Easy,
}
RETENTION: dict[ReturnPace, float] = {"often": 0.97, "balanced": 0.95, "rarely": 0.90}
SCHEDULERS = {
    pace: Scheduler(
        desired_retention=retention,
        learning_steps=(),
        relearning_steps=(),
        enable_fuzzing=False,
    )
    for pace, retention in RETENTION.items()
}


def learning_zone(connection: sqlite3.Connection) -> ZoneInfo:
    return ZoneInfo(read_settings(connection).learning_timezone)


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


def _scheduler(connection: sqlite3.Connection) -> Scheduler:
    return SCHEDULERS[read_settings(connection).return_pace]


def _review(
    connection: sqlite3.Connection,
    scheduler: Scheduler,
    fsrs_json: str | None,
    rating: str,
    instant: datetime,
) -> Card:
    card = Card.from_dict(json.loads(fsrs_json)) if fsrs_json else Card()
    updated, _ = scheduler.review_card(card, RATINGS[rating], review_datetime=instant)
    updated.due = max(updated.due, next_day_start(connection, instant))
    return updated


def schedule(
    connection: sqlite3.Connection,
    fsrs_json: str | None,
    rating: str,
    instant: datetime,
) -> tuple[str, str]:
    updated = _review(connection, _scheduler(connection), fsrs_json, rating, instant)
    return json.dumps(updated.to_dict()), updated.due.isoformat()


def return_days(
    connection: sqlite3.Connection,
    phrase: sqlite3.Row,
    instant: datetime,
) -> dict[str, int] | None:
    day = learning_day(connection, instant)
    if graded_today(connection, phrase, day):
        return None
    scheduler = _scheduler(connection)
    today = date.fromisoformat(day)
    days = {}
    for rating in RATINGS:
        due = _review(connection, scheduler, phrase["fsrs_json"], rating, instant).due
        days[rating] = (date.fromisoformat(learning_day(connection, due)) - today).days
    return days


def good_gaps(pace: ReturnPace, count: int) -> list[int]:
    scheduler, card, instant, gaps = SCHEDULERS[pace], Card(), datetime.now(UTC), []
    for _ in range(count):
        card, _ = scheduler.review_card(card, Rating.Good, review_datetime=instant)
        gaps.append((card.due - instant).days)
        instant = card.due
    return gaps
