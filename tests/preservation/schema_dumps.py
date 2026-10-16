"""Legacy SQLite schemas at every Alembic revision, recorded as plain SQL."""

import re
import sqlite3
import tempfile
from pathlib import Path

from alembic import command
from alembic.config import Config as AlembicConfig
from alembic.script import ScriptDirectory
from sqlalchemy import create_engine

import phrase_gym.storage

MIGRATIONS = Path(phrase_gym.storage.__file__).parent / "migrations"


def alembic_config() -> AlembicConfig:
    config = AlembicConfig()
    config.set_main_option("script_location", str(MIGRATIONS))
    return config


def revisions() -> list[str]:
    script = ScriptDirectory.from_config(alembic_config())
    return [revision.revision for revision in reversed(list(script.walk_revisions()))]


def schema_objects(connection: sqlite3.Connection) -> list[tuple[str, str, str]]:
    return list(
        connection.execute(
            "SELECT type, name, sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY type, name",
        ),
    )


def normalized(sql: str) -> str:
    collapsed = re.sub(r"\s+", " ", sql).strip()
    return re.sub(r"\s*([(),])\s*", r"\1", collapsed)


def alembic_schema(revision: str) -> list[tuple[str, str, str]]:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "legacy.sqlite3"
        engine = create_engine(f"sqlite:///{path.as_posix()}")
        config = alembic_config()
        with engine.begin() as connection:
            config.attributes["connection"] = connection
            command.upgrade(config, revision)
        engine.dispose()
        connection = sqlite3.connect(path)
        try:
            return schema_objects(connection)
        finally:
            connection.close()


def dump(revision: str) -> str:
    statements = []
    for _kind, _name, sql in alembic_schema(revision):
        lines = [line.rstrip() for line in sql.replace("\t", "    ").splitlines()]
        statements.append("\n".join(lines) + ";")
    statements.append(f"INSERT INTO alembic_version (version_num) VALUES ('{revision}');")
    return "\n\n".join(statements) + "\n"
