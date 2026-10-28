import os
import sqlite3
from collections.abc import Iterator
from contextlib import contextmanager
from datetime import datetime
from pathlib import Path

from phrase_gym.server.scheduling import next_day_start

SCHEMA = """
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS imports (
  package_id TEXT PRIMARY KEY, content_hash TEXT NOT NULL, payload TEXT NOT NULL,
  status TEXT NOT NULL, error TEXT, added_count INTEGER, created_at TEXT NOT NULL,
  result_id TEXT, result_hash TEXT
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
CREATE TABLE IF NOT EXISTS reviews (
  id TEXT PRIMARY KEY, attempt_id TEXT NOT NULL UNIQUE REFERENCES attempts(id),
  card_id TEXT NOT NULL REFERENCES cards(id), reviewed_at TEXT NOT NULL,
  learning_day TEXT NOT NULL, cue_snapshot TEXT NOT NULL, response_mode TEXT NOT NULL,
  help_before INTEGER NOT NULL, rating TEXT NOT NULL, due_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY CHECK(id=1),
  native_language TEXT NOT NULL, learning_timezone TEXT NOT NULL,
  card_language TEXT NOT NULL, help_language TEXT NOT NULL);
INSERT OR IGNORE INTO settings VALUES (1, 'ru', 'Europe/Belgrade', 'en', 'en');
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
            version = connection.execute(
                "SELECT value FROM meta WHERE key='schema_version'",
            ).fetchone()
            if version and version[0] not in ("1", "2"):
                raise RuntimeError(
                    f"Unsupported Phrase Gym data schema {version[0]}; restore a compatible backup",
                )
            connection.execute("INSERT OR IGNORE INTO meta VALUES ('schema_version', '2')")
            columns = {row[1] for row in connection.execute("PRAGMA table_info(imports)")}
            for name in ("result_id", "result_hash"):
                if name not in columns:
                    connection.execute(f"ALTER TABLE imports ADD COLUMN {name} TEXT")
            card_columns = {row[1] for row in connection.execute("PRAGMA table_info(cards)")}
            for name, definition in (
                ("due_at", "TEXT"),
                ("fsrs_json", "TEXT"),
                ("suspended", "INTEGER NOT NULL DEFAULT 0"),
                ("cue_index", "INTEGER NOT NULL DEFAULT 0"),
                ("exposed_day", "TEXT"),
            ):
                if name not in card_columns:
                    connection.execute(f"ALTER TABLE cards ADD COLUMN {name} {definition}")
            attempt_columns = {row[1] for row in connection.execute("PRAGMA table_info(attempts)")}
            for name, definition in (
                ("kind", "TEXT NOT NULL DEFAULT 'study'"),
                ("learning_day", "TEXT"),
                ("revealed_at", "TEXT"),
                ("response_mode", "TEXT"),
            ):
                if name not in attempt_columns:
                    connection.execute(f"ALTER TABLE attempts ADD COLUMN {name} {definition}")
            prior_study = connection.execute(
                "SELECT id, studied_at FROM cards WHERE studied_at IS NOT NULL AND due_at IS NULL",
            ).fetchall()
            for card in prior_study:
                due = next_day_start(connection, datetime.fromisoformat(card["studied_at"]))
                connection.execute(
                    "UPDATE cards SET due_at=? WHERE id=?",
                    (due.isoformat(), card["id"]),
                )
            connection.execute("UPDATE meta SET value='2' WHERE key='schema_version'")
            connection.execute("UPDATE imports SET status='pending' WHERE status='preparing'")

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
