from collections.abc import Callable
from datetime import UTC, datetime

from phrase_gym.server.contracts import DeletionResult
from phrase_gym.server.file_cleanup import FileCleanup
from phrase_gym.server.practice_views import clear_phrase, load_view
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
            # Takes live only in the phrase's own folder, which cleanup removes as a whole.
            directory = self.database.data_dir / "recordings" / phrase_id
            paths = [phrase["reference_path"], str(directory)]
            instant = self.clock().isoformat()
            connection.executemany(
                "INSERT OR IGNORE INTO cleanup_paths (path, phrase_id, recorded_at) VALUES (?,?,?)",
                [(path, phrase_id, instant) for path in paths],
            )
            connection.execute("DELETE FROM recordings WHERE phrase_id=?", (phrase_id,))
            if (view := load_view(connection)) and view["phrase_id"] == phrase_id:
                clear_phrase(connection)
            # Completed practice keeps its wording snapshot; the log's phrase reference is nulled.
            connection.execute("DELETE FROM phrases WHERE id=?", (phrase_id,))
            connection.execute(
                "INSERT INTO deleted_phrases (id, deleted_at) VALUES (?, ?)",
                (phrase_id, instant),
            )
            return phrase["text"]
