from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.contracts import DeletionResult
from phrase_gym.server.file_cleanup import FileCleanup
from phrase_gym.server.storage import Database


class PhraseDeletion:
    def __init__(
        self,
        database: Database,
        cleanup: FileCleanup,
        clock: Callable[[], datetime] | None = None,
    ):
        self.database = database
        self.cleanup = cleanup
        self.clock = clock or (lambda: datetime.now(UTC))

    def delete(self, phrase_id: str) -> DeletionResult | None:
        text = self.commit(phrase_id)
        if text is None:
            return None
        return DeletionResult(deleted_text=text, cleanup=self.cleanup.retry())

    def commit(self, phrase_id: str) -> str | None:
        with self.database.write() as connection:
            phrase = connection.execute(
                "SELECT text, reference_path FROM phrases WHERE id=?",
                (phrase_id,),
            ).fetchone()
            if phrase is None:
                return None
            takes = connection.execute(
                "SELECT path FROM recordings WHERE phrase_id=? ORDER BY rowid",
                (phrase_id,),
            ).fetchall()
            directory = self.database.data_dir / "recordings" / phrase_id
            paths = [phrase["reference_path"], *(row["path"] for row in takes), str(directory)]
            instant = self.clock().isoformat()
            connection.executemany(
                "INSERT OR IGNORE INTO cleanup_paths (path, phrase_id, recorded_at) VALUES (?,?,?)",
                [(path, phrase_id, instant) for path in paths],
            )
            for table in ("reviews", "recordings", "attempts"):
                connection.execute(f"DELETE FROM {table} WHERE phrase_id=?", (phrase_id,))  # noqa: S608
            connection.execute("DELETE FROM phrases WHERE id=?", (phrase_id,))
            connection.execute(
                "INSERT INTO deleted_phrases (id, deleted_at) VALUES (?, ?)",
                (phrase_id, instant),
            )
            return phrase["text"]
