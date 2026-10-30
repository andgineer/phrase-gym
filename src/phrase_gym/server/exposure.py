import sqlite3
from datetime import datetime

from phrase_gym.server.scheduling import learning_day


def expose_phrase(connection: sqlite3.Connection, phrase_id: str, instant: datetime) -> None:
    connection.execute(
        "UPDATE phrases SET exposed_day=? WHERE id=?",
        (learning_day(connection, instant), phrase_id),
    )
    connection.execute(
        "UPDATE attempts SET completed_at=? WHERE phrase_id=? AND kind='review' "
        "AND completed_at IS NULL",
        (instant.isoformat(), phrase_id),
    )
