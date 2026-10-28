import sqlite3
from datetime import datetime

from phrase_gym.server.scheduling import learning_day


def expose_card(connection: sqlite3.Connection, card_id: str, instant: datetime) -> None:
    connection.execute(
        "UPDATE cards SET exposed_day=? WHERE id=?",
        (learning_day(connection, instant), card_id),
    )
    connection.execute(
        "UPDATE attempts SET completed_at=? WHERE card_id=? AND kind='review' "
        "AND completed_at IS NULL",
        (instant.isoformat(), card_id),
    )
