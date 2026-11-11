import sqlite3
from collections.abc import Sequence
from dataclasses import dataclass

from phrase_gym.server.contracts import ActiveCardKind, CardKind, PracticeMode
from phrase_gym.server.practice_contracts import Rating


@dataclass(frozen=True, slots=True)
class ScheduleRecord:
    studied_at: str | None
    due_at: str | None
    last_review: str | None


@dataclass(frozen=True, slots=True)
class LessonScheduleRecord(ScheduleRecord):
    lesson_id: str


@dataclass(frozen=True, slots=True)
class PhraseRecord(ScheduleRecord):
    id: str
    lesson_id: str
    source_id: str
    item_id: str
    source_label: str
    source_link: str | None
    text: str
    explanation: str
    detailed_explanation: str | None
    cues_json: str
    hints_json: str
    contexts_json: str
    variants_json: str
    search_primary: str
    search_text: str
    reference_path: str
    created_at: str
    position: int
    fsrs_json: str | None
    last_cue: int | None


@dataclass(frozen=True, slots=True)
class ShownPhraseRecord(PhraseRecord):
    lesson_title: str


@dataclass(frozen=True, slots=True)
class LessonRecord:
    id: str
    source_id: str
    lesson_key: str
    title: str
    source_label: str
    search_text: str
    created_at: str
    position: int


@dataclass(frozen=True, slots=True)
class ViewRecord:
    id: int
    traversal_id: str
    learning_date: str
    origin: PracticeMode
    lesson_id: str | None
    skipped_json: str
    phrase_id: str | None
    view_token: str | None
    kind: ActiveCardKind | None
    practice: int
    revealed: int
    cue_index: int | None
    hint_index: int | None
    operations_json: str


@dataclass(frozen=True, slots=True)
class RecordingRecord:
    id: str
    phrase_id: str
    view_token: str
    captured_at: str
    mime_type: str
    duration_ms: int | None
    complete: int
    capture_settings_json: str
    path: str
    byte_length: int
    sha256: str
    mode: PracticeMode
    kind: CardKind
    cue_json: str | None
    hint: str | None
    answer_snapshot: str


@dataclass(frozen=True, slots=True)
class PracticeLogRecord:
    id: str
    phrase_id: str | None
    phrase_text: str
    lesson_title: str
    completed_at: str
    learning_date: str
    kind: CardKind
    rating: Rating | None
    skipped: int
    fsrs_updated: int
    mode: PracticeMode


@dataclass(frozen=True, slots=True)
class ImportRecord:
    package_id: str
    content_hash: str
    payload: str
    status: str
    error: str | None
    added_count: int | None
    created_at: str
    result_id: str | None
    result_hash: str | None
    kind: str


# The record's fields must equal the query's columns; a mismatch raises instead of dropping data.
def fetch_one[T](
    connection: sqlite3.Connection,
    record: type[T],
    sql: str,
    parameters: Sequence[object] = (),
) -> T | None:
    row = connection.execute(sql, parameters).fetchone()
    return None if row is None else record(**dict(row))


def fetch_all[T](
    connection: sqlite3.Connection,
    record: type[T],
    sql: str,
    parameters: Sequence[object] = (),
) -> list[T]:
    return [record(**dict(row)) for row in connection.execute(sql, parameters)]
