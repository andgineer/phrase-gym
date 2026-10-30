import sqlite3
from datetime import datetime

from phrase_gym.server.allowance import daily_progress
from phrase_gym.server.phrase_counts import SCHEDULE_COLUMNS, schedule_summary
from phrase_gym.server.practice_contracts import PracticeProgress, PracticeState
from phrase_gym.server.practice_views import load_view, practice_card, view_phrase


def _message(
    connection: sqlite3.Connection,
    view: sqlite3.Row,
    now: datetime,
    progress: PracticeProgress,
) -> str:
    lesson = view["lesson_id"]
    clause, parameters = (" WHERE lesson_id=?", (lesson,)) if lesson else ("", ())
    phrases = connection.execute(
        f"SELECT {SCHEDULE_COLUMNS} FROM phrases{clause}",  # noqa: S608
        parameters,
    ).fetchall()
    if not phrases:
        return "No phrases in this lesson." if lesson else "Add your first phrase batch to begin."
    summary = schedule_summary(connection, phrases, now)
    if view["origin"] == "extra":
        waiting = (summary.due_count and progress.due_allowance) or (
            summary.new_count and progress.new_allowance
        )
        finished = "Today's Extra practice is complete." if waiting else "Nothing waiting today."
    else:
        completed = progress.due_completed + progress.new_completed
        finished = "Today's Daily practice is complete." if completed else "Nothing waiting today."
    if summary.due_count or summary.new_count or not summary.next_due:
        return finished
    return f"{finished} Next due: {summary.next_due}."


def practice_state(connection: sqlite3.Connection, now: datetime, day: str) -> PracticeState:
    view = load_view(connection)
    progress = daily_progress(connection, day)
    if view is None or view["learning_date"] != day:
        return PracticeState(card=None, progress=progress, message=None)
    if view["phrase_id"]:
        card = practice_card(connection, view, view_phrase(connection, view))
        return PracticeState(card=card, progress=progress, message=None)
    return PracticeState(
        card=None,
        progress=progress,
        message=_message(connection, view, now, progress),
    )
