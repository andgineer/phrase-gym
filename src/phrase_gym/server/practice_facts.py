import sqlite3
from collections.abc import Callable
from datetime import UTC, date, datetime, time, timedelta

from phrase_gym.server.facts_contracts import (
    CumulativeWork,
    LibraryCounts,
    PracticeFacts,
    PracticeTotals,
    StudiedPage,
    StudiedPhrase,
)
from phrase_gym.server.import_notices import import_notices
from phrase_gym.server.phrase_counts import SCHEDULE_COLUMNS, schedule_summary
from phrase_gym.server.scheduling import learning_day, learning_zone
from phrase_gym.server.storage import Database
from phrase_gym.server.streak import practice_streak

LONG_TERM_STABILITY_DAYS = 21


def _day_bounds(connection: sqlite3.Connection, day: str) -> tuple[str, str]:
    zone = learning_zone(connection)
    start = datetime.combine(date.fromisoformat(day), time.min, zone)
    end = datetime.combine(date.fromisoformat(day) + timedelta(days=1), time.min, zone)
    return start.astimezone(UTC).isoformat(), end.astimezone(UTC).isoformat()


def _recordings(connection: sqlite3.Connection, day: str | None) -> int:
    if day is None:
        return connection.execute("SELECT COUNT(*) FROM recordings").fetchone()[0]
    # Takes carry the capturing device's offset; julianday compares them as instants.
    return connection.execute(
        "SELECT COUNT(*) FROM recordings "
        "WHERE julianday(captured_at)>=julianday(?) AND julianday(captured_at)<julianday(?)",
        _day_bounds(connection, day),
    ).fetchone()[0]


def _totals(connection: sqlite3.Connection, day: str | None) -> PracticeTotals:
    clause, parameters = ("WHERE learning_date=? ", (day,)) if day else ("", ())
    rows = connection.execute(
        "SELECT kind, rating, skipped, COUNT(*) AS count FROM practice_log "  # noqa: S608
        f"{clause}GROUP BY kind, rating, skipped",
        parameters,
    ).fetchall()
    ratings: dict[str, int] = {}
    for row in rows:
        if row["rating"]:
            ratings[row["rating"]] = ratings.get(row["rating"], 0) + row["count"]
    return PracticeTotals(
        completions=sum(row["count"] for row in rows),
        first_meetings=sum(row["count"] for row in rows if row["kind"] == "first_study"),
        new_attempts=sum(row["count"] for row in rows if row["kind"] == "new" and row["rating"]),
        recalls=sum(row["count"] for row in rows if row["kind"] == "recall" and row["rating"]),
        skipped=sum(row["count"] for row in rows if row["skipped"]),
        easy=ratings.get("easy", 0),
        got_it=ratings.get("got_it", 0),
        effort=ratings.get("effort", 0),
        not_yet=ratings.get("not_yet", 0),
        recordings=_recordings(connection, day),
    )


def _library(connection: sqlite3.Connection, now: datetime) -> LibraryCounts:
    rows = connection.execute(
        f"SELECT {SCHEDULE_COLUMNS} FROM phrases",  # noqa: S608
    ).fetchall()
    summary = schedule_summary(connection, rows, now)
    long_term = connection.execute(
        "SELECT COUNT(*) FROM phrases WHERE json_extract(fsrs_json, '$.stability')>=?",
        (LONG_TERM_STABILITY_DAYS,),
    ).fetchone()[0]
    return LibraryCounts(
        phrase_count=summary.phrase_count,
        due_count=summary.due_count,
        later_count=summary.phrase_count - summary.due_count - summary.new_count,
        new_count=summary.new_count,
        long_term_count=long_term,
    )


class PracticeFactsService:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def facts(self) -> PracticeFacts:
        with self.database.snapshot() as connection:
            now = self.clock()
            day = learning_day(connection, now)
            dates = [
                row[0]
                for row in connection.execute("SELECT DISTINCT learning_date FROM practice_log")
            ]
            return PracticeFacts(
                learning_date=day,
                streak=practice_streak(dates, day),
                today=_totals(connection, day),
                work=CumulativeWork(
                    **_totals(connection, None).model_dump(),
                    practice_days=len(dates),
                ),
                library=_library(connection, now),
                notices=import_notices(connection),
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
