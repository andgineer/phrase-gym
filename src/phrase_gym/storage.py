import os
import sqlite3
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

SCHEMA = """
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS imports (
  package_id TEXT PRIMARY KEY, content_hash TEXT NOT NULL, payload TEXT NOT NULL,
  status TEXT NOT NULL, error TEXT, added_count INTEGER, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS cards (
  id TEXT PRIMARY KEY, source_id TEXT NOT NULL, item_id TEXT NOT NULL,
  source_label TEXT NOT NULL, source_link TEXT, answer TEXT NOT NULL,
  explanation TEXT NOT NULL, cues_json TEXT NOT NULL, variants_json TEXT NOT NULL,
  detailed_explanation TEXT, reference_path TEXT NOT NULL, created_at TEXT NOT NULL,
  studied_at TEXT, UNIQUE(source_id, item_id)
);
CREATE TABLE IF NOT EXISTS attempts (
  id TEXT PRIMARY KEY, card_id TEXT NOT NULL REFERENCES cards(id), created_at TEXT NOT NULL,
  completed_at TEXT, answer_snapshot TEXT NOT NULL, explanation_snapshot TEXT NOT NULL,
  cue_snapshot TEXT NOT NULL, support_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS operations (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL, result_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS recordings (
  id TEXT PRIMARY KEY, card_id TEXT NOT NULL REFERENCES cards(id),
  attempt_id TEXT NOT NULL REFERENCES attempts(id), captured_at TEXT NOT NULL,
  mime_type TEXT NOT NULL, duration_ms INTEGER, complete INTEGER NOT NULL,
  capture_settings_json TEXT NOT NULL, path TEXT NOT NULL, byte_length INTEGER NOT NULL,
  sha256 TEXT NOT NULL
);
"""


class Database:
    def __init__(self, data_dir: Path):
        data_dir.mkdir(parents=True, mode=0o700, exist_ok=True)
        data_dir.chmod(0o700)
        self.data_dir = data_dir
        self.path = data_dir / "phrase-gym.sqlite3"
        if not self.path.exists():
            descriptor = os.open(self.path, os.O_CREAT | os.O_WRONLY | os.O_EXCL, 0o600)
            os.close(descriptor)
        self.path.chmod(0o600)
        with self.write() as connection:
            connection.executescript(SCHEMA)
            connection.execute("INSERT OR IGNORE INTO meta VALUES ('schema_version', '1')")
            connection.execute(
                "UPDATE imports SET status='error', "
                "error='Preparation was interrupted; retry this package.' "
                "WHERE status='preparing'",
            )

    def connect(self) -> sqlite3.Connection:
        connection = sqlite3.connect(self.path, timeout=5, check_same_thread=False)
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys=ON")
        connection.execute("PRAGMA journal_mode=WAL")
        self._secure_database_files()
        return connection

    def _secure_database_files(self) -> None:
        for suffix in ("", "-wal", "-shm"):
            path = Path(f"{self.path}{suffix}")
            if path.exists():
                path.chmod(0o600)

    @contextmanager
    def read(self) -> Iterator[sqlite3.Connection]:
        connection = self.connect()
        try:
            yield connection
        finally:
            connection.close()

    @contextmanager
    def write(self) -> Iterator[sqlite3.Connection]:
        connection = self.connect()
        try:
            connection.execute("BEGIN IMMEDIATE")
            yield connection
            connection.commit()
        except BaseException:
            connection.rollback()
            raise
        finally:
            connection.close()


def write_private(path: Path, content: bytes, *, exclusive: bool = True) -> None:
    flags = os.O_WRONLY | os.O_CREAT | (os.O_EXCL if exclusive else os.O_TRUNC)
    descriptor = os.open(path, flags, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(content)
    finally:
        os.close(descriptor)
