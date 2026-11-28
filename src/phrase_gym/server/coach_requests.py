import hashlib
import json
import sqlite3

from phrase_gym.server.coach_cards import coach_batch
from phrase_gym.server.coach_contracts import COACH_SOURCE_ID, CoachResult, CoachWork
from phrase_gym.server.import_notices import clear_notice, record_notice
from phrase_gym.server.importer import import_row, now, public_status, result_hash
from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.publication import publish_batch, reject_identity_conflicts
from phrase_gym.server.records import ImportRecord, fetch_one
from phrase_gym.server.reference_audio import ImportConflictError
from phrase_gym.server.settings_store import read_settings
from phrase_gym.server.storage import Database

PENDING = ("pending", "preparing")


def normalized(text: str) -> str:
    return " ".join(text.split()).casefold()


def request_id(text: str, learning_language: LearningLanguage) -> str:
    # English requests keep the identity they had before the language was recorded.
    key = normalized(text)
    if learning_language != "en":
        # German capitalisation tells words apart, as in Morgen (morning) and morgen (tomorrow).
        key = f"{learning_language}\n{' '.join(text.split())}"
    return "coach-" + hashlib.sha256(key.encode()).hexdigest()[:32]


def request_language(row: ImportRecord) -> LearningLanguage:
    return json.loads(row.payload).get("learning_language", "en")


def request_work(row: ImportRecord) -> CoachWork:
    """The languages captured at submission; older requests read as English, translation off."""
    payload = json.loads(row.payload)
    return CoachWork(
        package_id=row.package_id,
        text=payload["text"],
        learning_language=request_language(row),
        native_language=payload.get("native_language", "en"),
        translation=payload.get("translation", False),
    )


def coach_row(connection: sqlite3.Connection, package_id: str) -> ImportRecord:
    row = import_row(connection, package_id)
    if row is None or row.kind != "coach":
        raise KeyError(package_id)
    return row


def requeue(connection: sqlite3.Connection, package_id: str) -> None:
    connection.execute(
        "UPDATE imports SET status='pending', error=NULL, added_count=NULL, result_id=NULL, "
        "result_hash=NULL, retry_helps=1, dismissed=0, correction_json=NULL "
        "WHERE package_id=?",
        (package_id,),
    )
    clear_notice(connection, package_id)


def _in_library(connection: sqlite3.Connection, package_id: str) -> bool:
    return bool(
        connection.execute(
            "SELECT 1 FROM phrases WHERE source_id=? AND item_id=?",
            (COACH_SOURCE_ID, package_id),
        ).fetchone(),
    )


def _outcome(connection: sqlite3.Connection, row: ImportRecord) -> str:
    if row.status in PENDING:
        return "waiting"
    if row.status == "added" and _in_library(connection, row.package_id):
        return "in_library"
    if row.status == "error" and not row.retry_helps:
        # Unchanged text gets the same language decision, so the inbox shows it again.
        connection.execute("UPDATE imports SET dismissed=0 WHERE package_id=?", (row.package_id,))
        return "rejected"
    requeue(connection, row.package_id)
    return "saved"


class CoachRequests:
    """Typed Coach-field text waiting for, failing or finishing preparation."""

    def __init__(self, database: Database):
        self.database = database

    def submit(self, text: str, language: LearningLanguage) -> dict[str, str]:
        text = text.strip()
        if not text:
            raise ValueError("Type a phrase or word for Coach.")
        with self.database.write() as connection:
            package_id = request_id(text, language)
            row = import_row(connection, package_id)
            if row is not None and row.kind != "coach":
                raise ImportConflictError(f"{package_id} already names an external package.")
            if row is None:
                settings = read_settings(connection)
                payload = {
                    "text": text,
                    "learning_language": language,
                    "native_language": settings.native_language,
                    "translation": settings.translates(language),
                }
                connection.execute(
                    "INSERT INTO imports (package_id, content_hash, payload, status, created_at, "
                    "kind) VALUES (?, ?, ?, 'pending', ?, 'coach')",
                    (package_id, result_hash(normalized(text)), json.dumps(payload), now()),
                )
                outcome = "saved"
            else:
                outcome = _outcome(connection, row)
        return {"request_id": package_id, "outcome": outcome}

    def next_work(self) -> dict[str, object] | None:
        with self.database.write() as connection:
            row = fetch_one(
                connection,
                ImportRecord,
                "SELECT * FROM imports WHERE kind='coach' AND status IN ('pending', 'preparing') "
                "ORDER BY created_at LIMIT 1",
            )
            if row is None:
                return None
            connection.execute(
                "UPDATE imports SET status='preparing' WHERE package_id=?",
                (row.package_id,),
            )
            return request_work(row).model_dump()

    def fail(self, package_id: str, message: str, retry_helps: bool = True) -> dict[str, object]:
        with self.database.write() as connection:
            row = coach_row(connection, package_id)
            if row.status in PENDING:
                connection.execute(
                    "UPDATE imports SET status='error', error=?, retry_helps=?, dismissed=0 "
                    "WHERE package_id=?",
                    (message, int(retry_helps), package_id),
                )
                # Practice announces service failures; a language decision stays in Coach.
                if retry_helps:
                    record_notice(connection, package_id, now())
            return public_status(coach_row(connection, package_id))

    def publish(self, package_id: str, result: CoachResult) -> dict[str, object]:
        kind, card = ("phrase", result.phrase) if result.phrase else ("basic", result.basic)
        hashed = {
            kind: card.model_dump(mode="json") if card else None,
            "reference": result.reference,
        }
        if result.correction:
            # Only added when present, so results published before corrections keep their hash.
            hashed["correction"] = result.correction.model_dump(mode="json")
        digest = result_hash(hashed)
        with self.database.read() as connection:
            work = request_work(coach_row(connection, package_id))
        # The audio is decoded before the write lock; a request's languages never change.
        batch = coach_batch(work, result)
        correction = result.corrected()
        with self.database.write() as connection:
            row = coach_row(connection, package_id)
            if row.status == "added":
                if row.result_id != result.result_id or row.result_hash != digest:
                    raise ImportConflictError(
                        "A different result was already published for this request.",
                    )
                return public_status(row)
            if row.status not in PENDING:
                raise ImportConflictError("This request is not waiting for preparation.")
            reject_identity_conflicts(connection, batch)
            publish_batch(connection, self.database.data_dir, batch, (result.result_id, digest))
            connection.execute(
                "UPDATE imports SET correction_json=?, dismissed=0 WHERE package_id=?",
                (correction.model_dump_json() if correction else None, package_id),
            )
            return public_status(coach_row(connection, package_id))
