import hashlib
import json
import sqlite3

from phrase_gym.server.coach_contracts import (
    COACH_LESSON_ID,
    COACH_SOURCE_ID,
    COACH_SOURCE_LABEL,
)
from phrase_gym.server.import_contracts import ImportLesson, ImportPhrase, PhrasePackage
from phrase_gym.server.import_notices import clear_notice, record_notice
from phrase_gym.server.importer import import_row, now, public_status, result_hash
from phrase_gym.server.publication import publish_batch, reject_identity_conflicts
from phrase_gym.server.records import ImportRecord, fetch_one
from phrase_gym.server.reference_audio import ImportConflictError, decode_references
from phrase_gym.server.storage import Database

PENDING = ("pending", "preparing")


def normalized(text: str) -> str:
    return " ".join(text.split()).casefold()


def request_id(text: str) -> str:
    return "coach-" + hashlib.sha256(normalized(text).encode()).hexdigest()[:32]


def coach_package(package_id: str, phrase: ImportPhrase) -> PhrasePackage:
    return PhrasePackage(
        format="phrase-gym.package",
        version=2,
        package_id=package_id,
        source_id=COACH_SOURCE_ID,
        source_label=COACH_SOURCE_LABEL,
        learning_language="en",
        card_language="en",
        help_language="en",
        lessons=[
            ImportLesson(lesson_id=COACH_LESSON_ID, title=COACH_SOURCE_LABEL, phrases=[phrase]),
        ],
    )


def _in_library(connection: sqlite3.Connection, package_id: str) -> bool:
    return bool(
        connection.execute(
            "SELECT 1 FROM phrases WHERE source_id=? AND item_id=?",
            (COACH_SOURCE_ID, package_id),
        ).fetchone(),
    )


def _requeue(connection: sqlite3.Connection, package_id: str) -> None:
    connection.execute(
        "UPDATE imports SET status='pending', error=NULL, added_count=NULL, result_id=NULL, "
        "result_hash=NULL WHERE package_id=?",
        (package_id,),
    )
    clear_notice(connection, package_id)


class CoachRequests:
    """Typed Coach-field text waiting for, failing or finishing preparation."""

    def __init__(self, database: Database):
        self.database = database

    def submit(self, text: str) -> dict[str, str]:
        text = text.strip()
        if not text:
            raise ValueError("Type a phrase or word for Coach.")
        package_id = request_id(text)
        with self.database.write() as connection:
            row = import_row(connection, package_id)
            if row is not None and row.kind != "coach":
                raise ImportConflictError(f"{package_id} already names an external package.")
            if row is None:
                connection.execute(
                    "INSERT INTO imports (package_id, content_hash, payload, status, created_at, "
                    "kind) VALUES (?, ?, ?, 'pending', ?, 'coach')",
                    (package_id, result_hash(normalized(text)), json.dumps({"text": text}), now()),
                )
                outcome = "saved"
            elif row.status in PENDING:
                outcome = "waiting"
            elif row.status == "added" and _in_library(connection, package_id):
                outcome = "in_library"
            else:
                _requeue(connection, package_id)
                outcome = "saved"
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
            return {"package_id": row.package_id, "text": json.loads(row.payload)["text"]}

    def fail(self, package_id: str, message: str) -> dict[str, object]:
        with self.database.write() as connection:
            row = self._coach_row(connection, package_id)
            if row.status in PENDING:
                connection.execute(
                    "UPDATE imports SET status='error', error=? WHERE package_id=?",
                    (message, package_id),
                )
                record_notice(connection, package_id, now())
            return public_status(self._coach_row(connection, package_id))

    def publish(
        self,
        package_id: str,
        result_id: str,
        phrase: ImportPhrase,
        reference: str,
    ) -> dict[str, object]:
        if phrase.item_id != package_id:
            raise ImportConflictError(f"phrases.item_id must be {package_id}.")
        digest = result_hash({"phrase": phrase.model_dump(mode="json"), "reference": reference})
        package = coach_package(package_id, phrase)
        with self.database.write() as connection:
            row = self._coach_row(connection, package_id)
            if row.status == "added":
                if row.result_id != result_id or row.result_hash != digest:
                    raise ImportConflictError(
                        "A different result was already published for this request.",
                    )
                return public_status(row)
            if row.status not in PENDING:
                raise ImportConflictError("This request is not waiting for preparation.")
            decoded = decode_references(package, {package_id: reference})
            reject_identity_conflicts(connection, package)
            publish_batch(connection, self.database.data_dir, package, decoded, (result_id, digest))
            return public_status(self._coach_row(connection, package_id))

    @staticmethod
    def _coach_row(connection: sqlite3.Connection, package_id: str) -> ImportRecord:
        row = import_row(connection, package_id)
        if row is None or row.kind != "coach":
            raise KeyError(package_id)
        return row
