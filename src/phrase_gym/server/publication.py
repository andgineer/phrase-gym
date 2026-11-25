import json
import sqlite3
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from uuid import uuid4

from phrase_gym.server.import_contracts import ImportPhrase, PhrasePackage
from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.reference_audio import ImportConflictError
from phrase_gym.server.search_terms import search_key
from phrase_gym.server.storage import write_private


@dataclass(frozen=True, slots=True)
class BatchPhrase:
    lesson_key: str
    item_id: str
    # Phrase columns describing the content; identities and positions are added on publication.
    content: dict[str, object]
    audio: bytes


@dataclass(frozen=True, slots=True)
class Batch:
    package_id: str
    source_id: str
    source_label: str
    language: LearningLanguage
    lessons: list[tuple[str, str]]
    phrases: list[BatchPhrase]


def _content(package: PhrasePackage, phrase: ImportPhrase) -> dict[str, object]:
    cue_text = [part for cue in phrase.cues for part in (cue.situation, cue.partner_line)]
    return {
        "source_label": package.source_label,
        "source_link": str(package.source_link) if package.source_link else None,
        "text": phrase.text,
        "explanation": phrase.explanation,
        "detailed_explanation": phrase.detailed_explanation,
        "cues_json": json.dumps([cue.model_dump(exclude_none=True) for cue in phrase.cues]),
        "hints_json": json.dumps(phrase.recall_hints),
        "contexts_json": json.dumps(phrase.usage_contexts),
        "variants_json": "[]",
        "search_primary": search_key(phrase.text, *phrase.usage_contexts),
        "search_text": search_key(
            phrase.text,
            *phrase.usage_contexts,
            phrase.explanation,
            phrase.detailed_explanation,
            *cue_text,
        ),
    }


def package_batch(package: PhrasePackage, decoded: dict[str, bytes]) -> Batch:
    return Batch(
        package_id=package.package_id,
        source_id=package.source_id,
        source_label=package.source_label,
        language=package.learning_language,
        lessons=[(lesson.lesson_id, lesson.title) for lesson in package.lessons],
        phrases=[
            BatchPhrase(
                lesson.lesson_id,
                phrase.item_id,
                _content(package, phrase),
                decoded[phrase.item_id],
            )
            for lesson, phrase in package.phrases()
        ],
    )


def reject_identity_conflicts(connection: sqlite3.Connection, batch: Batch) -> None:
    for key, title in batch.lessons:
        existing = connection.execute(
            "SELECT title FROM lessons WHERE source_id=? AND lesson_key=?",
            (batch.source_id, key),
        ).fetchone()
        if existing and existing["title"] != title:
            raise ImportConflictError(
                f"lessons.{key}: this source lesson already exists as "
                f"“{existing['title']}”. Use that title or a new lesson_id; nothing was imported.",
            )
    for phrase in batch.phrases:
        existing = connection.execute(
            "SELECT text FROM phrases WHERE source_id=? AND item_id=?",
            (batch.source_id, phrase.item_id),
        ).fetchone()
        if existing:
            raise ImportConflictError(
                f"phrases.{phrase.item_id}: this source item is already in the Library as "
                f"“{existing['text']}”. Delete that phrase first to import a replacement; "
                "nothing was imported.",
            )


def _lesson_ids(connection: sqlite3.Connection, batch: Batch, created_at: str) -> dict[str, str]:
    identities: dict[str, str] = {}
    for position, (key, title) in enumerate(batch.lessons):
        row = connection.execute(
            "SELECT id FROM lessons WHERE source_id=? AND lesson_key=?",
            (batch.source_id, key),
        ).fetchone()
        if row:
            identities[key] = row["id"]
            continue
        identity = str(uuid4())
        connection.execute(
            "INSERT INTO lessons (id,source_id,lesson_key,title,source_label,search_text,"
            "created_at,position) VALUES (?,?,?,?,?,?,?,?)",
            (
                identity,
                batch.source_id,
                key,
                title,
                batch.source_label,
                search_key(title, batch.source_label),
                created_at,
                position,
            ),
        )
        identities[key] = identity
    return identities


def _insert_phrase(connection: sqlite3.Connection, row: dict[str, object]) -> None:
    columns = ",".join(row)
    placeholders = ",".join(f":{name}" for name in row)
    connection.execute(f"INSERT INTO phrases ({columns}) VALUES ({placeholders})", row)  # noqa: S608


def publish_batch(
    connection: sqlite3.Connection,
    data_dir: Path,
    batch: Batch,
    result: tuple[str, str],
) -> None:
    result_id, result_hash = result
    references = data_dir / "references"
    references.mkdir(mode=0o700, exist_ok=True)
    references.chmod(0o700)
    created: list[Path] = []
    created_at = datetime.now(UTC).isoformat()
    try:
        lesson_ids = _lesson_ids(connection, batch, created_at)
        for position, phrase in enumerate(batch.phrases):
            # Fresh identities keep a replacement import away from paths awaiting cleanup.
            phrase_id = str(uuid4())
            destination = references / f"{phrase_id}.wav"
            created.append(destination)
            write_private(destination, phrase.audio)
            _insert_phrase(
                connection,
                {
                    "id": phrase_id,
                    "lesson_id": lesson_ids[phrase.lesson_key],
                    "source_id": batch.source_id,
                    "item_id": phrase.item_id,
                    "language": batch.language,
                    **phrase.content,
                    "reference_path": str(destination),
                    "created_at": created_at,
                    "position": position,
                },
            )
        connection.execute(
            "UPDATE imports SET status='added', error=NULL, added_count=?, "
            "result_id=?, result_hash=? WHERE package_id=?",
            (len(created), result_id, result_hash, batch.package_id),
        )
    except BaseException:
        for path in created:
            path.unlink(missing_ok=True)
        raise
