import sqlite3

from phrase_gym.server.facts_contracts import ImportNotice
from phrase_gym.server.storage import Database


def record_notice(connection: sqlite3.Connection, package_id: str, at: str) -> None:
    connection.execute(
        "INSERT OR REPLACE INTO import_notices (package_id, created_at) VALUES (?, ?)",
        (package_id, at),
    )


def clear_notice(connection: sqlite3.Connection, package_id: str) -> None:
    connection.execute("DELETE FROM import_notices WHERE package_id=?", (package_id,))


def import_notices(connection: sqlite3.Connection) -> list[ImportNotice]:
    rows = connection.execute(
        "SELECT i.package_id, json_extract(i.payload, '$.source_label') AS source_label, "
        "i.error FROM import_notices n JOIN imports i USING (package_id) "
        "ORDER BY n.created_at, n.package_id",
    ).fetchall()
    return [
        ImportNotice(
            package_id=row["package_id"],
            source_label=row["source_label"],
            error=row["error"] or "",
        )
        for row in rows
    ]


class ImportNotices:
    def __init__(self, database: Database):
        self.database = database

    def dismiss(self, package_id: str) -> None:
        with self.database.write() as connection:
            clear_notice(connection, package_id)
