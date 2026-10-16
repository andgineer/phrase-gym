import sqlite3

import pytest

from tests.preservation import schema_dumps
from tests.preservation.export import schema_path


@pytest.mark.parametrize("revision", schema_dumps.revisions())
def test_committed_schema_builds_the_alembic_database(revision):
    connection = sqlite3.connect(":memory:")
    try:
        connection.executescript(schema_path(revision).read_text(encoding="utf-8"))
        committed = schema_dumps.schema_objects(connection)
        version = connection.execute("SELECT version_num FROM alembic_version").fetchall()
    finally:
        connection.close()
    expected = schema_dumps.alembic_schema(revision)
    assert [(kind, name, schema_dumps.normalized(sql)) for kind, name, sql in committed] == [
        (kind, name, schema_dumps.normalized(sql)) for kind, name, sql in expected
    ]
    assert version == [(revision,)]


def test_every_legacy_revision_has_a_fixture():
    assert schema_dumps.revisions() == ["0001", "0002", "0003", "0004", "0005"]
