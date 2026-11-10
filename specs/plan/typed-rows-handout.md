# Handout: typed records for database rows

A self-contained task for a fresh session. Read [AGENTS.md](../../AGENTS.md) first; its rules and done gate apply. This is a behavior-preserving refactor, not a delivery stage: implement it directly in one session, without the orchestration workflow, delegated agents or a review loop.

## Goal

Rows read from SQLite are `sqlite3.Row` objects read as `row["column"]`. The type checker cannot see a misspelled column name there, so only a failing test reveals it. After this task, every row that is read whole or passed between functions is a typed record read as `row.column`, and Pyrefly rejects a misspelled name at commit.

Nothing the learner sees changes. No endpoint, response, schema, SQL statement text, transaction boundary or dependency changes.

## Decisions already made (do not reopen)

- SQL stays hand-written. No ORM, query builder or new dependency.
- Records are frozen, slotted standard-library dataclasses in one new module, built from a row by keyword so that a record's fields must equal the query's columns.
- Stored values keep their stored form: JSON columns stay `str`, flags stay `int`. Do not decode JSON or convert flags inside records.
- Writes (`INSERT`, `UPDATE`, `DELETE`) are untouched.

## Scope rule

Convert a read when either holds:

1. it selects a whole row (`SELECT *`, `SELECT phrases.*, …`, or the `SCHEDULE_COLUMNS` projection), or
2. its row is passed to, or returned from, another function.

Leave a read unchanged when it names a few columns and consumes them in the same function, or reads a single value by position (`fetchone()[0]`, `SELECT 1 …`). Those sit next to their own SQL text. The unchanged reads are in `allowance.py`, `deletion.py` (its own phrase lookup), `file_cleanup.py`, `import_notices.py`, `publication.py`, `settings_store.py`, `storage.py`, `completion.py` and `extra.py` (the `operations` lookup), `library.py` (the search listing and the reference path) and `practice_facts.py` (counts, totals, distinct dates).

## Step 1: add `src/phrase_gym/server/records.py`

Create the file with exactly this content. It was checked against this repository: Pyrefly and Ruff pass, and each whole-table record matches the columns of a freshly created database.

```python
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
```

Notes on the types:

- `origin`, `mode`, `kind` and `rating` use the contract literals because the API models they feed require them. Dataclasses do not validate at runtime; these annotations state what the application itself writes.
- `ViewRecord.kind` is `ActiveCardKind | None`: an upgrade step already cleared stored `first_study` views. `PracticeLogRecord.kind` and `RecordingRecord.kind` are `CardKind` because history keeps `first_study`.

## Step 2: add `tests/test_records.py`

Cover three behaviors, using a `Database` on `tmp_path` (see `tests/test_storage.py` for how tests construct one):

1. Parametrized over `phrases → PhraseRecord`, `lessons → LessonRecord`, `current_view → ViewRecord`, `recordings → RecordingRecord`, `practice_log → PracticeLogRecord`, `imports → ImportRecord`: the set of `PRAGMA table_info(<table>)` column names equals the set of `dataclasses.fields(<record>)` names. This is the guard that makes a future schema step fail loudly until its record is updated.
2. `fetch_all` raises `TypeError` when the query's columns differ from the record's fields (for example `LessonRecord` with `SELECT id FROM lessons` after inserting one lesson row, or any query returning a column the record lacks).
3. `fetch_one` returns `None` when no row matches.

## Step 3: convert the readers, one module at a time

For each read in scope, replace `connection.execute(sql, params).fetchone()` with `fetch_one(connection, Record, sql, params)` and `.fetchall()` with `fetch_all(...)`. Keep the SQL string and parameters byte-for-byte. Replace `sqlite3.Row` annotations with the record type and `row["name"]` with `row.name`. Remove `import sqlite3` only where nothing else in the module uses it.

Work in this order and run `uv run pyrefly check` after each module. Pyrefly drives the migration: once a function returns a record, every remaining `row["name"]` on it is reported as `Cannot index into …`, and every caller still annotated `sqlite3.Row` is reported as a bad argument. Fix what it lists, then move on. `uv run pytest` must pass after each module.

| Module | Reads to convert | Record |
|---|---|---|
| `practice_views.py` | `load_view`; `view_phrase` (joined with the lesson title) | `ViewRecord`; `ShownPhraseRecord` |
| `scheduling.py` | no query; `effective_due` and `graded_today` take `ScheduleRecord`, `return_days` takes `PhraseRecord` | — |
| `phrase_counts.py` | no query; `schedule_summary` takes `Iterable[ScheduleRecord]` | — |
| `practice_selection.py` | due phrases, new phrases | `PhraseRecord` |
| `practice_state.py` | the `SCHEDULE_COLUMNS` read | `ScheduleRecord` |
| `phrase_practice.py` | the phrase lookup | `PhraseRecord` |
| `completion.py`, `daily.py`, `current_practice.py`, `deletion.py`, `view_actions.py` | no query in scope; they consume the view and phrase records | — |
| `practice_facts.py` | the `SCHEDULE_COLUMNS` read; the studied page | `ScheduleRecord`; `PracticeLogRecord` |
| `lesson_choices.py` | both lesson reads; the `lesson_id` + `SCHEDULE_COLUMNS` read | `LessonRecord`; `LessonScheduleRecord` |
| `library.py` | phrase details (joined); phrase recordings | `ShownPhraseRecord`; `RecordingRecord` |
| `recording_views.py` | view recordings; `saved_recording` takes the record | `RecordingRecord` |
| `recording_store.py` | the existing-recording lookup | `RecordingRecord` |
| `importer.py` | all five `imports` reads; `_public` takes the record | `ImportRecord` |

## Known findings and their required resolution

These appeared when the central module was converted as a trial. Resolve them exactly this way.

1. **A current view without a shown card.** `ViewRecord.phrase_id`, `view_token` and `kind` are nullable, and `view_phrase` can now return `None`. The old code accepted this silently. Where Pyrefly reports `… | None` is not assignable (in `practice_views.py`: the return of `view_phrase`, and `view_token`/`kind` in `practice_card`; in `completion.py`: `kind` for the receipt), add an explicit guard that raises `StaleViewError("This card is no longer current. Open Daily to continue.")` before the value is used. Put the guard once per function, at the top. These guards are unreachable in correct operation (deletion clears the view in the same transaction), so add no tests for them.
2. **Column chosen by name in `view_actions.py`.** `phrase[values_column]` and `view[column]` look a column up by a string from `CHOICES`. Do not replace them with `getattr`. Select explicitly: `phrase.cues_json if kind == "cue" else phrase.hints_json` and `view.cue_index if kind == "cue" else view.hint_index`. Keep `column` for the `UPDATE` statement; drop `values_column` from `CHOICES` if nothing else uses it.
3. **A narrow row passed to a helper in `recording_store.py`.** `_require_shown_support(upload, phrase)` receives a three-column row. Leave that query unchanged and change the helper to take `cues_json: str, hints_json: str`, passing `phrase["cues_json"]` and `phrase["hints_json"]`.
4. **`sqlite3.Row` built by a join.** `ShownPhraseRecord` matches only `SELECT phrases.*, lessons.title AS lesson_title`. Do not reuse it for another projection; if a query's columns differ, the scope rule decides whether it needs its own record.

Forbidden ways to silence the checker: `cast`, `# type: ignore`, `# pyrefly: ignore`, `Any` annotations, `assert` for narrowing, `getattr`/`__getitem__` shims on records, giving record fields default values, or loosening a record's field type to make a call site pass. If a finding fits none of the four resolutions above, stop and report it with the Pyrefly message instead of improvising.

## Step 4: record the decision

In [architecture.md](../architecture.md), in the paragraph of "Responsibilities" that begins "Use one application database", append:

> SQL is hand-written against SQLite without an ORM, because each workflow's transaction takes its write lock up front and the queries rely on SQLite's own date and JSON functions. Rows read whole or passed between functions are typed records whose fields equal the query's columns, so the type checker rejects a misspelled column.

Then delete this handout and remove its entry from the [implementation plan](implementation-plan.md), in the same commit as the code.

## Acceptance

- `grep -rn "sqlite3.Row" src` prints only the `row_factory` line in `storage.py`.
- `git diff` shows no change to any SQL string, to `src/phrase_gym/server/schema/`, to `*contracts.py`, to the generated API contract, to `frontend/` or to `pyproject.toml`/`uv.lock`.
- No existing test is edited, skipped or weakened; the only test change is the new `tests/test_records.py`.
- Every production module stays under the size limit enforced by the pre gate.
- A deliberate check, reverted afterwards: change one `phrase.text` to `phrase.txt` and confirm `uv run pyrefly check` fails.

## Gate and delivery

Follow the done gate in AGENTS.md for an implementation change:

1. Before editing, inspect GitHub Actions for the current remote commit (`gh run list --branch main --limit 5`).
2. Iterate with `uv run pyrefly check` and `uv run pytest`.
3. Before the full browser suite, run one existing browser test per engine with `--max-failures=1`; stop and report if a browser cannot launch.
4. Run `uv run inv pre`, then `uv run inv test`, and read their complete results.
5. Commit to `main` with one commit, push, wait for the workflows on that exact commit and inspect every job. Never use `--no-verify` and never force-push.
6. Report the commit, the run link and the CI outcome separately from the local checks.
