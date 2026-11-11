import sqlite3
import stat
import sys
from contextlib import closing
from pathlib import Path

import pytest

from phrase_gym.server import main as cli
from phrase_gym.server import schema_upgrade
from phrase_gym.server.contracts import Settings
from phrase_gym.server.schema_upgrade import STEPS, statements
from phrase_gym.server.settings_store import read_settings
from phrase_gym.server.storage import Database

SCHEMA_5_ROWS = """
INSERT INTO meta VALUES ('schema_version', '5');
INSERT INTO imports (package_id, content_hash, payload, status, error, created_at)
  VALUES ('course-v1', 'hash', '{"source_label": "Course"}', 'error', 'Voice missing',
          '2026-10-30T10:00:00+00:00');
INSERT INTO lessons VALUES ('lesson-1', 'source', 'unit-1', 'Meetings', 'Course', 'meetings',
  '2026-10-30T10:00:00+00:00', 0);
INSERT INTO phrases (id, lesson_id, source_id, item_id, source_label, text, explanation,
  cues_json, hints_json, contexts_json, variants_json, search_primary, search_text,
  reference_path, created_at, position, studied_at, due_at)
  VALUES ('phrase-1', 'lesson-1', 'source', 'item-1', 'Course', 'That works for me.',
  'Accept a time.', '[]', '[]', '[]', '[]', 'that works', 'that works for me',
  'references/phrase-1.wav', '2026-10-30T10:00:00+00:00', 0,
  '2026-10-31T09:00:00+00:00', '2026-11-03T09:00:00+00:00');
INSERT INTO practice_log VALUES ('log-1', 'phrase-1', 'That works for me.', 'Meetings',
  '2026-10-31T09:00:00+00:00', '2026-10-31', 'first_study', NULL, 0, 1, 'daily');
UPDATE settings SET daily_due_limit=9;
"""


def schema_5_directory(path: Path) -> Path:
    path.mkdir()
    with closing(sqlite3.connect(path / "phrase-gym.sqlite3")) as connection:
        connection.execute("PRAGMA journal_mode=WAL")
        for statement in statements(STEPS[5] + SCHEMA_5_ROWS):
            connection.execute(statement)
        connection.commit()
    return path


def learner_rows(connection: sqlite3.Connection) -> list[tuple[object, ...]]:
    return [
        *connection.execute("SELECT package_id, status, payload FROM imports"),
        *connection.execute("SELECT id, title FROM lessons"),
        *connection.execute("SELECT id, text, due_at FROM phrases"),
        *connection.execute("SELECT id, phrase_id, learning_date FROM practice_log"),
    ]


def version(connection: sqlite3.Connection) -> str:
    return connection.execute("SELECT value FROM meta WHERE key='schema_version'").fetchone()[0]


def table_names(connection: sqlite3.Connection) -> set[str]:
    return {row[0] for row in connection.execute("SELECT name FROM sqlite_master")}


def test_schema_5_database_is_upgraded_in_place_after_a_backup(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    directory = schema_5_directory(tmp_path / "data")
    with closing(sqlite3.connect(directory / "phrase-gym.sqlite3")) as connection:
        before = learner_rows(connection)
    database = Database(directory)
    with database.read() as connection:
        assert version(connection) == schema_upgrade.SCHEMA_VERSION
        assert [tuple(row) for row in learner_rows(connection)] == before
        [error] = connection.execute("SELECT error FROM imports").fetchone()
        assert "saved without reference audio" in error
        assert "import_notices" in table_names(connection)
        assert read_settings(connection) == Settings(
            native_language="ru",
            learning_timezone="Europe/Belgrade",
            card_language="en",
            help_language="en",
            daily_due_limit=9,
            daily_new_limit=5,
            return_pace="balanced",
            weekly_target=None,
            weekly_target_paused=False,
        )
    [backup] = directory.glob("phrase-gym-schema-5-*.sqlite3")
    assert capsys.readouterr().out == (
        f"Upgraded the Phrase Gym database from schema 5 to {schema_upgrade.SCHEMA_VERSION}; "
        f"the previous version is saved as {backup}\n"
    )
    assert stat.S_IMODE(backup.stat().st_mode) == 0o600
    assert not list(directory.glob("phrase-gym-schema-5-*.sqlite3-*"))
    with closing(sqlite3.connect(backup)) as copy:
        assert version(copy) == "5"
        assert learner_rows(copy) == before
        assert copy.execute("SELECT daily_due_limit FROM settings").fetchone()[0] == 9
        assert "import_notices" not in table_names(copy)
    Database(directory)
    assert len(list(directory.glob("phrase-gym-schema-*.sqlite3"))) == 1
    assert capsys.readouterr().out == ""


def test_failed_upgrade_step_leaves_the_database_at_its_old_version(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    directory = schema_5_directory(tmp_path / "data")
    with closing(sqlite3.connect(directory / "phrase-gym.sqlite3")) as connection:
        before = learner_rows(connection)
    monkeypatch.setitem(
        STEPS, 6, STEPS[6] + "INSERT INTO import_notices VALUES ('missing-column');"
    )
    with pytest.raises(sqlite3.OperationalError):
        Database(directory)
    with closing(sqlite3.connect(directory / "phrase-gym.sqlite3")) as connection:
        assert version(connection) == "5"
        assert "import_notices" not in table_names(connection)
        assert learner_rows(connection) == before
    monkeypatch.undo()
    with Database(directory).read() as connection:
        assert version(connection) == schema_upgrade.SCHEMA_VERSION


def test_fresh_database_is_built_from_every_step_without_a_backup(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    with Database(tmp_path / "fresh").read() as connection:
        assert version(connection) == schema_upgrade.SCHEMA_VERSION
        assert {"imports", "phrases", "import_notices"} <= table_names(connection)
        assert read_settings(connection).daily_due_limit == 20
    assert not list((tmp_path / "fresh").glob("phrase-gym-schema-*"))
    assert capsys.readouterr().out == ""


def test_statements_keep_semicolons_inside_literals() -> None:
    script = "INSERT INTO t VALUES ('a;b');\n\nCREATE TABLE u (x);\n"
    assert [part.strip() for part in statements(script)] == [
        "INSERT INTO t VALUES ('a;b');",
        "CREATE TABLE u (x);",
    ]


def test_cli_starts_on_an_upgradable_directory(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    directory = schema_5_directory(tmp_path / "data")
    started = []
    monkeypatch.setattr(cli.uvicorn, "run", lambda *args, **kwargs: started.append(args))
    monkeypatch.setattr(sys, "argv", ["phrase-gym", "run", "--data-dir", str(directory)])
    cli.main()
    assert len(started) == 1


SCHEMA_8_IMPORTS = """
INSERT INTO meta VALUES ('schema_version', '8');
INSERT INTO imports (package_id, content_hash, payload, status, created_at)
  VALUES ('waiting', 'h1', '{"source_label": "Waiting course"}', 'pending',
          '2026-11-01T10:00:00+00:00'),
         ('claimed', 'h2', '{"source_label": "Claimed course"}', 'preparing',
          '2026-11-01T10:01:00+00:00'),
         ('failed', 'h3', '{"source_label": "Failed course"}', 'error',
          '2026-11-01T10:02:00+00:00'),
         ('added', 'h4', '{"source_label": "Added course"}', 'added',
          '2026-11-01T10:03:00+00:00');
UPDATE imports SET error='Voice missing' WHERE package_id='failed';
INSERT INTO import_notices VALUES ('failed', '2026-11-01T11:00:00+00:00');
"""


def test_audio_less_package_submissions_become_kept_failures_with_notices(tmp_path: Path) -> None:
    directory = tmp_path / "data"
    directory.mkdir()
    with closing(sqlite3.connect(directory / "phrase-gym.sqlite3")) as connection:
        script = "".join(STEPS[version] for version in range(5, 9)) + SCHEMA_8_IMPORTS
        for statement in statements(script):
            connection.execute(statement)
        connection.commit()
    with Database(directory).read() as connection:
        rows = {
            row["package_id"]: (row["kind"], row["status"], row["payload"])
            for row in connection.execute("SELECT * FROM imports")
        }
        errors = dict(connection.execute("SELECT package_id, error FROM imports"))
        noticed = {row[0] for row in connection.execute("SELECT package_id FROM import_notices")}
    assert rows == {
        "waiting": ("package", "error", '{"source_label": "Waiting course"}'),
        "claimed": ("package", "error", '{"source_label": "Claimed course"}'),
        "failed": ("package", "error", '{"source_label": "Failed course"}'),
        "added": ("package", "added", '{"source_label": "Added course"}'),
    }
    assert errors["added"] is None
    for package_id in ("waiting", "claimed", "failed"):
        assert "without reference audio" in errors[package_id]
        assert "new package_id" in errors[package_id]
    assert noticed == {"waiting", "claimed", "failed"}


def test_coach_failures_keep_their_reason_when_old_package_failures_are_explained(
    tmp_path: Path,
) -> None:
    directory = tmp_path / "data"
    directory.mkdir()
    rows = """
INSERT INTO meta VALUES ('schema_version', '9');
INSERT INTO imports (package_id, content_hash, payload, status, error, created_at, kind)
  VALUES ('old-package', 'h1', '{"source_label": "Course"}', 'error', 'Piper is not configured',
          '2026-11-01T10:00:00+00:00', 'package'),
         ('coach-1', 'h2', '{"text": "by and large"}', 'error', 'Codex is not logged in',
          '2026-11-01T10:01:00+00:00', 'coach');
"""
    with closing(sqlite3.connect(directory / "phrase-gym.sqlite3")) as connection:
        script = "".join(STEPS[version] for version in range(5, 10)) + rows
        for statement in statements(script):
            connection.execute(statement)
        connection.commit()
    with Database(directory).read() as connection:
        errors = dict(connection.execute("SELECT package_id, error FROM imports"))
        payload = connection.execute(
            "SELECT payload FROM imports WHERE package_id='old-package'"
        ).fetchone()[0]
    assert errors["coach-1"] == "Codex is not logged in"
    assert "saved without reference audio" in errors["old-package"]
    assert payload == '{"source_label": "Course"}'
