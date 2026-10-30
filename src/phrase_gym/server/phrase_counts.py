import sqlite3
from collections.abc import Iterable
from datetime import datetime

from phrase_gym.server.contracts import PracticeCounts
from phrase_gym.server.scheduling import effective_due


def schedule_summary(
    connection: sqlite3.Connection,
    rows: Iterable[sqlite3.Row],
    now: datetime,
) -> tuple[PracticeCounts, datetime | None]:
    phrases = list(rows)
    due_times = [effective_due(connection, row) for row in phrases if row["studied_at"]]
    later = [when for when in due_times if when > now]
    counts = PracticeCounts(
        phrase_count=len(phrases),
        due_count=sum(when <= now for when in due_times),
        new_count=len(phrases) - len(due_times),
    )
    return counts, min(later) if later else None
