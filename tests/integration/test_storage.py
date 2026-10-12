from alembic.autogenerate import compare_metadata
from alembic.migration import MigrationContext
import json
from pathlib import Path
from alembic import command
from alembic.config import Config as AlembicConfig
from sqlalchemy import text
from phrase_gym.storage import Base, Database, AppSettings
from phrase_gym.settings import SettingsService


def test_migration_matches_models_and_sqlite_settings(config):
    database = Database(config.data_dir)
    try:
        database.migrate()
        database.migrate()
        with database.engine.connect() as connection:
            differences = compare_metadata(MigrationContext.configure(connection), Base.metadata)
            assert differences == []
            assert connection.execute(text("PRAGMA foreign_keys")).scalar() == 1
            assert connection.execute(text("PRAGMA journal_mode")).scalar() == "wal"
    finally:
        database.close()


def test_legacy_settings_migrate_without_losing_native_or_ui_preferences(config):
    database = Database(config.data_dir)
    try:
        migration = AlembicConfig()
        migration.set_main_option(
            "script_location",
            str(Path(__file__).resolve().parents[2] / "src/phrase_gym/migrations"),
        )
        with database.engine.begin() as connection:
            migration.attributes["connection"] = connection
            command.upgrade(migration, "0001")
        with database.write() as session:
            session.add(
                AppSettings(
                    id=1,
                    payload=json.dumps(
                        {
                            "native_language": "ru",
                            "ui_language_mode": "learning",
                            "last_learning_language": "sr",
                        }
                    ),
                )
            )
        database.migrate()
        database.migrate()
        assert SettingsService(database).settings().model_dump() == {
            "native_language": "ru",
            "ui_language_mode": "learning",
        }
        with database.read() as session:
            assert json.loads(session.get(AppSettings, 1).payload) == {
                "native_language": "ru",
                "ui_language_mode": "learning",
            }
    finally:
        database.close()
