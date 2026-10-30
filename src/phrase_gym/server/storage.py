import os
import sqlite3
from collections.abc import Iterator
from contextlib import closing, contextmanager
from pathlib import Path

SCHEMA_VERSION = "4"
DATABASE_NAME = "phrase-gym.sqlite3"
SCHEMA = """
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE imports (
  package_id TEXT PRIMARY KEY, content_hash TEXT NOT NULL, payload TEXT NOT NULL,
  status TEXT NOT NULL, error TEXT, added_count INTEGER, created_at TEXT NOT NULL,
  result_id TEXT, result_hash TEXT
);
CREATE TABLE lessons (
  id TEXT PRIMARY KEY, source_id TEXT NOT NULL, lesson_key TEXT NOT NULL,
  title TEXT NOT NULL, source_label TEXT NOT NULL, search_text TEXT NOT NULL,
  created_at TEXT NOT NULL, position INTEGER NOT NULL, UNIQUE(source_id, lesson_key)
);
CREATE TABLE phrases (
  id TEXT PRIMARY KEY, lesson_id TEXT NOT NULL REFERENCES lessons(id),
  source_id TEXT NOT NULL, item_id TEXT NOT NULL, source_label TEXT NOT NULL,
  source_link TEXT, text TEXT NOT NULL, explanation TEXT NOT NULL,
  detailed_explanation TEXT, cues_json TEXT NOT NULL, hints_json TEXT NOT NULL,
  contexts_json TEXT NOT NULL, variants_json TEXT NOT NULL,
  search_primary TEXT NOT NULL, search_text TEXT NOT NULL, reference_path TEXT NOT NULL,
  created_at TEXT NOT NULL, position INTEGER NOT NULL, studied_at TEXT, due_at TEXT,
  fsrs_json TEXT, last_review TEXT, last_cue INTEGER,
  UNIQUE(source_id, item_id)
);
CREATE INDEX phrases_by_lesson ON phrases(lesson_id, created_at, position);
CREATE INDEX phrases_by_due ON phrases(due_at);
CREATE TABLE operations (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL, request_json TEXT NOT NULL,
  result_json TEXT NOT NULL
);
CREATE TABLE recordings (
  id TEXT PRIMARY KEY, phrase_id TEXT NOT NULL REFERENCES phrases(id),
  view_token TEXT NOT NULL, captured_at TEXT NOT NULL,
  mime_type TEXT NOT NULL, duration_ms INTEGER, complete INTEGER NOT NULL,
  capture_settings_json TEXT NOT NULL, path TEXT NOT NULL, byte_length INTEGER NOT NULL,
  sha256 TEXT NOT NULL, mode TEXT NOT NULL, kind TEXT NOT NULL, cue_json TEXT, hint TEXT,
  answer_snapshot TEXT NOT NULL
);
CREATE INDEX recordings_by_phrase ON recordings(phrase_id, captured_at);
CREATE INDEX recordings_by_view ON recordings(view_token, captured_at);
CREATE TABLE practice_log (
  id TEXT PRIMARY KEY, phrase_id TEXT REFERENCES phrases(id) ON DELETE SET NULL,
  phrase_text TEXT NOT NULL, lesson_title TEXT NOT NULL, completed_at TEXT NOT NULL,
  learning_date TEXT NOT NULL, kind TEXT NOT NULL, rating TEXT, skipped INTEGER NOT NULL,
  fsrs_updated INTEGER NOT NULL, mode TEXT NOT NULL
);
CREATE INDEX practice_log_by_date ON practice_log(learning_date, mode, kind);
CREATE INDEX practice_log_by_phrase ON practice_log(phrase_id);
CREATE TABLE extra_targets (
  learning_date TEXT PRIMARY KEY, due_target INTEGER NOT NULL, new_target INTEGER NOT NULL
);
CREATE TABLE current_view (
  id INTEGER PRIMARY KEY CHECK(id=1), traversal_id TEXT NOT NULL,
  learning_date TEXT NOT NULL, origin TEXT NOT NULL, lesson_id TEXT,
  skipped_json TEXT NOT NULL, phrase_id TEXT, view_token TEXT, kind TEXT,
  revealed INTEGER NOT NULL, cue_index INTEGER, hint_index INTEGER,
  operations_json TEXT NOT NULL
);
CREATE TABLE deleted_phrases (id TEXT PRIMARY KEY, deleted_at TEXT NOT NULL);
CREATE TABLE cleanup_paths (
  path TEXT PRIMARY KEY, phrase_id TEXT NOT NULL, recorded_at TEXT NOT NULL, error TEXT
);
CREATE TABLE settings (id INTEGER PRIMARY KEY CHECK(id=1),
  native_language TEXT NOT NULL, learning_timezone TEXT NOT NULL,
  card_language TEXT NOT NULL, help_language TEXT NOT NULL,
  daily_due_limit INTEGER NOT NULL, daily_new_limit INTEGER NOT NULL);
INSERT INTO settings VALUES (1, 'ru', 'Europe/Belgrade', 'en', 'en', 20, 5);
"""


class UnsupportedSchemaError(RuntimeError):
    pass


def _schema_version(connection: sqlite3.Connection) -> str | None:
    tables = {row[0] for row in connection.execute("SELECT name FROM sqlite_master")}
    if not tables:
        return None
    if "meta" not in tables:
        return "unknown"
    row = connection.execute("SELECT value FROM meta WHERE key='schema_version'").fetchone()
    return row[0] if row else "unknown"


def existing_schema_version(path: Path) -> str | None:
    if not path.exists() or path.stat().st_size == 0:
        return None
    # Immutable inspection creates no WAL sidecars in a directory this version cannot use.
    with closing(sqlite3.connect(f"{path.as_uri()}?immutable=1", uri=True)) as connection:
        return _schema_version(connection)


def unsupported(data_dir: Path, version: str) -> UnsupportedSchemaError:
    return UnsupportedSchemaError(
        f"{data_dir} holds Phrase Gym data schema {version}; this version needs schema "
        f"{SCHEMA_VERSION}. Start with a new empty --data-dir or restore a compatible "
        "backup. The existing directory was not changed.",
    )


def require_compatible(data_dir: Path) -> None:
    version = existing_schema_version(data_dir / DATABASE_NAME)
    if version not in (None, SCHEMA_VERSION):
        raise unsupported(data_dir, version)


class Database:
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.path = data_dir / DATABASE_NAME
        require_compatible(data_dir)
        data_dir.mkdir(parents=True, mode=0o700, exist_ok=True)
        data_dir.chmod(0o700)
        if not self.path.exists():
            descriptor = os.open(self.path, os.O_CREAT | os.O_WRONLY | os.O_EXCL, 0o600)
            os.close(descriptor)
        self.path.chmod(0o600)
        with self.write() as connection:
            current = _schema_version(connection)
            if current is None:
                for statement in SCHEMA.split(";"):
                    if statement.strip():
                        connection.execute(statement)
                connection.execute(
                    "INSERT INTO meta VALUES ('schema_version', ?)",
                    (SCHEMA_VERSION,),
                )
            elif current != SCHEMA_VERSION:
                raise unsupported(data_dir, current)
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


def owned_path(data_dir: Path, path: str) -> Path:
    resolved = Path(path).resolve()
    if not resolved.is_relative_to(data_dir.resolve()):
        raise ValueError(f"{path} is outside the application data directory")
    return resolved
