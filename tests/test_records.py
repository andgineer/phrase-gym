import dataclasses
from pathlib import Path

import pytest

from phrase_gym.server.records import (
    ImportRecord,
    LessonRecord,
    PhraseRecord,
    PracticeLogRecord,
    RecordingRecord,
    ViewRecord,
    fetch_all,
    fetch_one,
)
from phrase_gym.server.storage import Database


@pytest.mark.parametrize(
    ("table", "record"),
    [
        ("phrases", PhraseRecord),
        ("lessons", LessonRecord),
        ("current_view", ViewRecord),
        ("recordings", RecordingRecord),
        ("practice_log", PracticeLogRecord),
        ("imports", ImportRecord),
    ],
)
def test_whole_table_record_matches_a_fresh_database(
    tmp_path: Path, table: str, record: type
) -> None:
    with Database(tmp_path).read() as connection:
        columns = {row["name"] for row in connection.execute(f"PRAGMA table_info({table})")}
    assert columns == {field.name for field in dataclasses.fields(record)}


def insert_lesson(database: Database) -> None:
    with database.write() as connection:
        connection.execute(
            "INSERT INTO lessons (id, source_id, lesson_key, title, source_label, search_text, "
            "created_at, position) VALUES ('lesson-1', 'source', 'key', 'Title', 'Label', "
            "'title', '2026-01-01T00:00:00Z', 0)"
        )


def test_whole_lesson_row_becomes_a_record(tmp_path: Path) -> None:
    database = Database(tmp_path)
    insert_lesson(database)
    with database.read() as connection:
        lessons = fetch_all(connection, LessonRecord, "SELECT * FROM lessons")
    assert [(lesson.id, lesson.title, lesson.position) for lesson in lessons] == [
        ("lesson-1", "Title", 0)
    ]


@pytest.mark.parametrize(
    "sql",
    ["SELECT id FROM lessons", "SELECT *, 'extra' AS surplus FROM lessons"],
    ids=["missing column", "surplus column"],
)
def test_columns_that_differ_from_the_record_raise(tmp_path: Path, sql: str) -> None:
    database = Database(tmp_path)
    insert_lesson(database)
    with database.read() as connection, pytest.raises(TypeError):
        fetch_all(connection, LessonRecord, sql)


def test_fetch_one_without_a_match_returns_none(tmp_path: Path) -> None:
    with Database(tmp_path).read() as connection:
        found = fetch_one(connection, LessonRecord, "SELECT * FROM lessons WHERE id=?", ("absent",))
    assert found is None
