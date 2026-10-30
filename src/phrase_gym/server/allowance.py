import sqlite3

from phrase_gym.server.practice_contracts import PracticeProgress

ALLOWANCE_MODES = ("daily", "extra")


def _effective(limit: int, target: int | None) -> int:
    # A zero setting disables its category even when an earlier Extra target exists.
    return 0 if limit == 0 else max(limit, target or 0)


def daily_progress(connection: sqlite3.Connection, day: str) -> PracticeProgress:
    due_limit, new_limit = connection.execute(
        "SELECT daily_due_limit, daily_new_limit FROM settings WHERE id=1",
    ).fetchone()
    target = connection.execute(
        "SELECT due_target, new_target FROM extra_targets WHERE learning_date=?",
        (day,),
    ).fetchone()
    completed = dict(
        connection.execute(
            "SELECT kind, COUNT(*) FROM practice_log WHERE learning_date=? "
            "AND mode IN (?, ?) GROUP BY kind",
            (day, *ALLOWANCE_MODES),
        ).fetchall(),
    )
    return PracticeProgress(
        due_completed=completed.get("recall", 0),
        due_allowance=_effective(due_limit, target["due_target"] if target else None),
        new_completed=completed.get("first_study", 0),
        new_allowance=_effective(new_limit, target["new_target"] if target else None),
    )
