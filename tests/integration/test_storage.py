from alembic.autogenerate import compare_metadata
from alembic.migration import MigrationContext
from sqlalchemy import text
from phrase_gym.storage import Base, Database


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
