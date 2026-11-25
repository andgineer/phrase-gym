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
from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.phrase_counts import SCHEDULE_COLUMNS, schedule_summary
from phrase_gym.server.records import PracticeLogRecord, ScheduleRecord, fetch_all
from phrase_gym.server.scheduling import learning_day, learning_zone
from phrase_gym.server.settings_store import read_settings
from phrase_gym.server.storage import Database
from phrase_gym.server.streak import practice_streak, practice_week

LONG_TERM_STABILITY_DAYS = 21


def _day_bounds(connection: sqlite3.Connection, day: str) -> tuple[str, str]:
    zone = learning_zone(connection)
    start = datetime.combine(date.fromisoformat(day), time.min, zone)
    end = datetime.combine(date.fromisoformat(day) + timedelta(days=1), time.min, zone)
    return start.astimezone(UTC).isoformat(), end.astimezone(UTC).isoformat()


def _recordings(connection: sqlite3.Connection, today: tuple[str, LearningLanguage] | None) -> int:
    if today is None:
        return connection.execute("SELECT COUNT(*) FROM recordings").fetchone()[0]
    day, language = today
    # Takes carry the capturing device's offset; julianday compares them as instants.
    return connection.execute(
        "SELECT COUNT(*) FROM recordings JOIN phrases ON phrases.id=recordings.phrase_id "
        "WHERE phrases.language=? AND julianday(captured_at)>=julianday(?) "
        "AND julianday(captured_at)<julianday(?)",
        (language, *_day_bounds(connection, day)),
    ).fetchone()[0]


def _totals(
    connection: sqlite3.Connection,
    today: tuple[str, LearningLanguage] | None,
) -> PracticeTotals:
    """Today's figures for one language, or everything done in every language."""
    clause, parameters = ("AND learning_date=? AND language=? ", today) if today else ("", ())
    rows = connection.execute(
        "SELECT kind, rating, COUNT(*) AS count FROM practice_log "  # noqa: S608
        f"WHERE skipped=0 {clause}GROUP BY kind, rating",
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
        skipped=0,
        easy=ratings.get("easy", 0),
        got_it=ratings.get("got_it", 0),
        effort=ratings.get("effort", 0),
        not_yet=ratings.get("not_yet", 0),
        recordings=_recordings(connection, today),
    )


def _library(
    connection: sqlite3.Connection,
    now: datetime,
    language: LearningLanguage,
) -> LibraryCounts:
    rows = fetch_all(
        connection,
        ScheduleRecord,
        f"SELECT {SCHEDULE_COLUMNS} FROM phrases WHERE language=?",  # noqa: S608
        (language,),
    )
    summary = schedule_summary(connection, rows, now)
    long_term = connection.execute(
        "SELECT COUNT(*) FROM phrases WHERE language=? "
        "AND json_extract(fsrs_json, '$.stability')>=?",
        (language, LONG_TERM_STABILITY_DAYS),
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

    def facts(self, language: LearningLanguage) -> PracticeFacts:
        with self.database.snapshot() as connection:
            now = self.clock()
            day = learning_day(connection, now)
            # The streak, the weekly target and Progress count practice in every language.
            dates = [
                row[0]
                for row in connection.execute(
                    "SELECT DISTINCT learning_date FROM practice_log WHERE skipped=0",
                )
            ]
            return PracticeFacts(
                learning_date=day,
                streak=practice_streak(dates, day),
                week=practice_week(dates, day, read_settings(connection)),
                today=_totals(connection, (day, language)),
                work=CumulativeWork(
                    **_totals(connection, None).model_dump(),
                    practice_days=len(dates),
                ),
                library=_library(connection, now, language),
                notices=import_notices(connection),
            )

    def studied(
        self,
        language: LearningLanguage,
        learning_date: str,
        offset: int,
        limit: int,
    ) -> StudiedPage:
        with self.database.read() as connection:
            rows = fetch_all(
                connection,
                PracticeLogRecord,
                "SELECT * FROM practice_log WHERE language=? AND learning_date=? AND skipped=0 "
                "ORDER BY completed_at, rowid LIMIT ? OFFSET ?",
                (language, learning_date, limit + 1, offset),
            )
        return StudiedPage(
            items=[
                StudiedPhrase(
                    completion_id=row.id,
                    phrase_id=row.phrase_id,
                    text=row.phrase_text,
                    lesson_title=row.lesson_title,
                    mode=row.mode,
                    kind=row.kind,
                    rating=row.rating,
                    skipped=bool(row.skipped),
                )
                for row in rows[:limit]
            ],
            next_offset=offset + limit if len(rows) > limit else None,
        )
