import sqlite3

from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.practice_contracts import PracticeProgress
from phrase_gym.server.settings_store import read_settings

ALLOWANCE_MODES = ("daily", "extra")


def _effective(limit: int, target: int | None) -> int:
    # A zero setting disables its category even when an earlier Extra target exists.
    return 0 if limit == 0 else max(limit, target or 0)


def _stored_targets(
    connection: sqlite3.Connection,
    day: str,
    language: LearningLanguage,
) -> sqlite3.Row | None:
    return connection.execute(
        "SELECT due_target, new_target FROM extra_targets WHERE learning_date=? AND language=?",
        (day, language),
    ).fetchone()


def daily_progress(
    connection: sqlite3.Connection,
    day: str,
    language: LearningLanguage,
) -> PracticeProgress:
    settings = read_settings(connection)
    due_limit, new_limit = settings.daily_due_limit, settings.daily_new_limit
    target = _stored_targets(connection, day, language)
    completed = dict(
        connection.execute(
            "SELECT kind, COUNT(*) FROM practice_log WHERE language=? AND learning_date=? "
            "AND skipped=0 AND mode IN (?, ?) GROUP BY kind",
            (language, day, *ALLOWANCE_MODES),
        ).fetchall(),
    )
    return PracticeProgress(
        due_completed=completed.get("recall", 0),
        due_allowance=_effective(due_limit, target["due_target"] if target else None),
        new_completed=completed.get("first_study", 0) + completed.get("new", 0),
        new_allowance=_effective(new_limit, target["new_target"] if target else None),
    )


def _extended(limit: int, completed: int, target: int) -> int:
    # Zero disables the category, so it keeps any earlier target and gets no extension.
    if limit == 0:
        return target
    return max(limit if completed < limit else completed + limit, target)


def extend_targets(
    connection: sqlite3.Connection,
    day: str,
    language: LearningLanguage,
) -> tuple[int, int]:
    settings = read_settings(connection)
    due_limit, new_limit = settings.daily_due_limit, settings.daily_new_limit
    stored = _stored_targets(connection, day, language)
    progress = daily_progress(connection, day, language)
    targets = (
        _extended(due_limit, progress.due_completed, stored["due_target"] if stored else 0),
        _extended(new_limit, progress.new_completed, stored["new_target"] if stored else 0),
    )
    connection.execute(
        "INSERT OR REPLACE INTO extra_targets (learning_date, language, due_target, new_target) "
        "VALUES (?,?,?,?)",
        (day, language, *targets),
    )
    return targets
