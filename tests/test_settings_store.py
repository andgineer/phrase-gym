import json
from pathlib import Path

from phrase_gym.server.settings_store import read_settings, write_settings
from phrase_gym.server.storage import Database


def test_missing_settings_read_as_defaults_and_retired_ones_are_ignored(tmp_path: Path) -> None:
    database = Database(tmp_path / "data")
    with database.write() as connection:
        stored = json.loads(connection.execute("SELECT document FROM settings").fetchone()[0])
        assert "return_pace" not in stored
        connection.execute(
            "UPDATE settings SET document=?",
            (json.dumps(stored | {"retired_setting": True}),),
        )
    with database.read() as connection:
        settings = read_settings(connection)
    assert settings.return_pace == "balanced"
    assert settings.daily_due_limit == 20


def test_saving_keeps_newer_settings_and_drops_retired_ones(tmp_path: Path) -> None:
    database = Database(tmp_path / "data")
    with database.write() as connection:
        connection.execute("UPDATE settings SET document=json_set(document, '$.retired', 1)")
        changed = read_settings(connection).model_copy(
            update={"return_pace": "rarely", "daily_new_limit": 3},
        )
        write_settings(connection, changed)
    with database.read() as connection:
        assert read_settings(connection) == changed
        stored = json.loads(connection.execute("SELECT document FROM settings").fetchone()[0])
    assert stored["return_pace"] == "rarely" and "retired" not in stored
