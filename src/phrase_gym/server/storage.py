import os
import sqlite3
from collections.abc import Iterator
from contextlib import closing, contextmanager
from pathlib import Path

from phrase_gym.server.schema_upgrade import (
    BASELINE_VERSION,
    SCHEMA_VERSION,
    UPGRADABLE_VERSIONS,
    apply_steps,
    backup_before_upgrade,
)

DATABASE_NAME = "phrase-gym.sqlite3"


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
        f"{data_dir} holds Phrase Gym data schema {version}; this version upgrades schema "
        f"{BASELINE_VERSION} or later up to {SCHEMA_VERSION}. Use the Phrase Gym version "
        "that wrote it, restore a compatible backup or choose another --data-dir. "
        "The existing directory was not changed.",
    )


def require_compatible(data_dir: Path) -> None:
    version = existing_schema_version(data_dir / DATABASE_NAME)
    if version is not None and version not in UPGRADABLE_VERSIONS:
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
        with self.read() as connection:
            found = _schema_version(connection)
        backup = None
        if found not in (None, SCHEMA_VERSION):
            backup = backup_before_upgrade(self.path, str(found))
        with self.write() as connection:
            current = _schema_version(connection)
            if current is not None and current not in UPGRADABLE_VERSIONS:
                raise unsupported(data_dir, current)
            if current != SCHEMA_VERSION:
                apply_steps(connection, current)
            connection.execute("UPDATE imports SET status='pending' WHERE status='preparing'")
        if backup is not None:
            print(
                f"Upgraded the Phrase Gym database from schema {found} to {SCHEMA_VERSION}; "
                f"the previous version is saved as {backup}",
                flush=True,
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
    def snapshot(self) -> Iterator[sqlite3.Connection]:
        # A deferred transaction takes no write lock; under WAL its first read fixes the snapshot.
        connection = self.connect()
        try:
            connection.execute("BEGIN DEFERRED")
            yield connection
        finally:
            connection.rollback()
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
