import hashlib
import sqlite3
import sys
from pathlib import Path

import pytest

from phrase_gym.server import main as cli
from phrase_gym.server.storage import SCHEMA_VERSION, Database, UnsupportedSchemaError


def earlier_directory(path: Path, script: str) -> dict[str, str]:
    path.mkdir()
    connection = sqlite3.connect(path / "phrase-gym.sqlite3")
    connection.execute("PRAGMA journal_mode=WAL")
    connection.executescript(script)
    connection.commit()
    connection.close()
    return fingerprint(path)


def fingerprint(path: Path) -> dict[str, str]:
    return {
        item.name: hashlib.sha256(item.read_bytes()).hexdigest() for item in sorted(path.iterdir())
    }


EARLIER = (
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);"
    "INSERT INTO meta VALUES ('schema_version','2');"
    "CREATE TABLE cards (id TEXT PRIMARY KEY, answer TEXT);"
    "INSERT INTO cards VALUES ('old-card', 'Old answer');"
)


def test_earlier_schema_is_rejected_without_changing_the_directory(tmp_path: Path) -> None:
    directory = tmp_path / "earlier"
    before = earlier_directory(directory, EARLIER)
    with pytest.raises(UnsupportedSchemaError, match="schema 2") as raised:
        Database(directory)
    assert "new empty --data-dir" in str(raised.value)
    assert fingerprint(directory) == before


def test_unversioned_database_is_rejected(tmp_path: Path) -> None:
    directory = tmp_path / "unversioned"
    before = earlier_directory(directory, "CREATE TABLE cards (id TEXT);")
    with pytest.raises(UnsupportedSchemaError, match="schema unknown"):
        Database(directory)
    assert fingerprint(directory) == before


def test_fresh_directory_creates_and_reopens_current_schema(tmp_path: Path) -> None:
    Database(tmp_path / "fresh")
    with Database(tmp_path / "fresh").read() as connection:
        version = connection.execute("SELECT value FROM meta WHERE key='schema_version'")
        assert version.fetchone()[0] == SCHEMA_VERSION
        tables = {row[0] for row in connection.execute("SELECT name FROM sqlite_master")}
    assert {"phrases", "lessons", "cleanup_paths", "deleted_phrases"} <= tables
    assert "cards" not in tables


def test_cli_reports_earlier_schema_without_starting(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    directory = tmp_path / "earlier"
    before = earlier_directory(directory, EARLIER)
    started = []
    monkeypatch.setattr(cli.uvicorn, "run", lambda *args, **kwargs: started.append(args))
    monkeypatch.setattr(sys, "argv", ["phrase-gym", "run", "--data-dir", str(directory)])
    with pytest.raises(SystemExit) as stopped:
        cli.main()
    assert stopped.value.code == 2
    assert "schema 2" in capsys.readouterr().err
    assert started == []
    assert fingerprint(directory) == before
