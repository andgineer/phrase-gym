import sqlite3
from collections.abc import Callable
from datetime import UTC, date, datetime, time, timedelta

from phrase_gym.server.facts_contracts import (
    CumulativeWork,
    NextReturn,
    PracticeFacts,
    StudiedPage,
    StudiedPhrase,
    TodayCounts,
)
from phrase_gym.server.phrase_counts import SCHEDULE_COLUMNS
from phrase_gym.server.scheduling import effective_due, learning_day, learning_zone
from phrase_gym.server.storage import Database
from phrase_gym.server.streak import practice_streak


def _day_bounds(connection: sqlite3.Connection, day: str) -> tuple[str, str]:
    zone = learning_zone(connection)
    start = datetime.combine(date.fromisoformat(day), time.min, zone)
    end = datetime.combine(date.fromisoformat(day) + timedelta(days=1), time.min, zone)
    return start.astimezone(UTC).isoformat(), end.astimezone(UTC).isoformat()


def _today_counts(connection: sqlite3.Connection, day: str) -> TodayCounts:
    rows = connection.execute(
        "SELECT kind, rating, skipped, COUNT(*) AS count FROM practice_log "
        "WHERE learning_date=? GROUP BY kind, rating, skipped",
        (day,),
    ).fetchall()
    ratings = {row["rating"]: row["count"] for row in rows if row["rating"]}
    # Takes carry the capturing device's offset; julianday compares them as instants.
    recordings = connection.execute(
        "SELECT COUNT(*) FROM recordings "
        "WHERE julianday(captured_at)>=julianday(?) AND julianday(captured_at)<julianday(?)",
        _day_bounds(connection, day),
    ).fetchone()[0]
    return TodayCounts(
        completions=sum(row["count"] for row in rows),
        first_meetings=sum(row["count"] for row in rows if row["kind"] == "first_study"),
        recalls=sum(row["count"] for row in rows if row["kind"] == "recall"),
        skipped=sum(row["count"] for row in rows if row["skipped"]),
        got_it=ratings.get("got_it", 0),
        effort=ratings.get("effort", 0),
        not_yet=ratings.get("not_yet", 0),
        recordings=recordings,
    )


def _next_return(connection: sqlite3.Connection, day: str) -> NextReturn | None:
    rows = connection.execute(
        f"SELECT {SCHEDULE_COLUMNS} FROM phrases WHERE studied_at IS NOT NULL",  # noqa: S608
    ).fetchall()
    due_days = [learning_day(connection, effective_due(connection, row)) for row in rows]
    if not due_days:
        return None
    # Overdue phrases are offered again today, never on an earlier date.
    first = max(min(due_days), day)
    return NextReturn(learning_date=first, phrase_count=sum(due <= first for due in due_days))


class PracticeFactsService:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def facts(self) -> PracticeFacts:
        with self.database.snapshot() as connection:
            day = learning_day(connection, self.clock())
            dates = [
                row[0]
                for row in connection.execute("SELECT DISTINCT learning_date FROM practice_log")
            ]
            completions, first_meetings = connection.execute(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE kind='first_study') FROM practice_log",
            ).fetchone()
            return PracticeFacts(
                learning_date=day,
                streak=practice_streak(dates, day),
                today=_today_counts(connection, day),
                work=CumulativeWork(
                    completions=completions,
                    first_meetings=first_meetings,
                    practice_days=len(dates),
                ),
                next_return=_next_return(connection, day),
            )

    def studied(self, learning_date: str, offset: int, limit: int) -> StudiedPage:
        with self.database.read() as connection:
            rows = connection.execute(
                "SELECT * FROM practice_log WHERE learning_date=? "
                "ORDER BY completed_at, rowid LIMIT ? OFFSET ?",
                (learning_date, limit + 1, offset),
            ).fetchall()
        return StudiedPage(
            items=[
                StudiedPhrase(
                    completion_id=row["id"],
                    phrase_id=row["phrase_id"],
                    text=row["phrase_text"],
                    lesson_title=row["lesson_title"],
                    mode=row["mode"],
                    kind=row["kind"],
                    rating=row["rating"],
                    skipped=bool(row["skipped"]),
                )
                for row in rows[:limit]
            ],
            next_offset=offset + limit if len(rows) > limit else None,
        )
