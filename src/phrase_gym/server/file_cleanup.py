from pathlib import Path

from phrase_gym.server.contracts import CleanupStatus
from phrase_gym.server.storage import Database, owned_path


def remove_owned_path(path: Path) -> None:
    if path.is_dir():
        for leftover in path.glob("*.pending"):
            leftover.unlink(missing_ok=True)
        path.rmdir()
    else:
        path.unlink(missing_ok=True)


class FileCleanup:
    def __init__(self, database: Database):
        self.database = database

    def status(self) -> CleanupStatus:
        with self.database.read() as connection:
            pending = connection.execute("SELECT COUNT(*) FROM cleanup_paths").fetchone()[0]
            failure = connection.execute(
                "SELECT error FROM cleanup_paths WHERE error IS NOT NULL ORDER BY rowid LIMIT 1",
            ).fetchone()
        error = None
        if failure:
            error = (
                f"{pending} file(s) of deleted phrases could not be removed yet: {failure[0]}. "
                "Fix the problem, then retry cleanup."
            )
        return CleanupStatus(pending=pending, error=error)

    def retry(self) -> CleanupStatus:
        with self.database.read() as connection:
            paths = [
                row[0]
                for row in connection.execute("SELECT path FROM cleanup_paths ORDER BY rowid")
            ]
        for path in paths:
            try:
                remove_owned_path(owned_path(self.database.data_dir, path))
            except (OSError, ValueError) as error:
                with self.database.write() as connection:
                    connection.execute(
                        "UPDATE cleanup_paths SET error=? WHERE path=?",
                        (str(error)[:500], path),
                    )
                continue
            with self.database.write() as connection:
                connection.execute("DELETE FROM cleanup_paths WHERE path=?", (path,))
        return self.status()
