import shutil
from pathlib import Path

from phrase_gym.server.contracts import CleanupStatus
from phrase_gym.server.storage import Database, owned_path


def remove_owned_path(path: Path) -> None:
    if path.is_dir():
        # Only a deleted phrase's own take folder is removed whole, including orphaned files.
        shutil.rmtree(path)
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

    def _owned(self, path: str, phrase_id: str) -> Path:
        if Path(path).is_symlink():
            raise ValueError(f"{path} is a symbolic link")
        resolved = owned_path(self.database.data_dir, path)
        if resolved.is_dir():
            recordings = (self.database.data_dir / "recordings").resolve()
            if resolved.parent != recordings or resolved.name != phrase_id:
                raise ValueError(f"{path} is not the deleted phrase's recordings folder")
            with self.database.read() as connection:
                if connection.execute("SELECT 1 FROM phrases WHERE id=?", (phrase_id,)).fetchone():
                    raise ValueError(f"{path} belongs to a phrase that still exists")
        return resolved

    def retry(self) -> CleanupStatus:
        with self.database.read() as connection:
            paths = connection.execute(
                "SELECT path, phrase_id FROM cleanup_paths ORDER BY rowid",
            ).fetchall()
        for path, phrase_id in paths:
            try:
                remove_owned_path(self._owned(path, phrase_id))
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
