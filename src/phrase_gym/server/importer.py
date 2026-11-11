import hashlib
import json
import sqlite3
from datetime import UTC, datetime

from phrase_gym.server.coach_contracts import COACH_SOURCE_ID
from phrase_gym.server.import_contracts import PhrasePackage
from phrase_gym.server.publication import publish_batch, reject_identity_conflicts
from phrase_gym.server.records import ImportRecord, fetch_one
from phrase_gym.server.reference_audio import ImportConflictError, decode_references
from phrase_gym.server.storage import Database


def now() -> str:
    return datetime.now(UTC).isoformat()


def result_hash(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def import_row(connection: sqlite3.Connection, package_id: str) -> ImportRecord | None:
    return fetch_one(
        connection,
        ImportRecord,
        "SELECT * FROM imports WHERE package_id=?",
        (package_id,),
    )


def public_status(row: ImportRecord) -> dict[str, object]:
    return {
        "package_id": row.package_id,
        "status": row.status,
        "error": row.error,
        "added_count": row.added_count,
    }


class Importer:
    def __init__(self, database: Database):
        self.database = database

    def submit(self, package: PhrasePackage, references: dict[str, str]) -> dict[str, object]:
        if package.source_id == COACH_SOURCE_ID:
            raise ImportConflictError(
                f"source_id {COACH_SOURCE_ID} is reserved for the Coach field; use another.",
            )
        payload = package.model_dump_json(exclude_none=True)
        decoded = decode_references(package, references)
        digest = result_hash({"package": payload, "references": references})
        with self.database.write() as connection:
            previous = import_row(connection, package.package_id)
            if previous and previous.content_hash != digest:
                raise ImportConflictError(
                    "This package_id has different content; use a new ID.",
                )
            if previous is None:
                reject_identity_conflicts(connection, package)
                connection.execute(
                    "INSERT INTO imports (package_id, content_hash, payload, status, created_at, "
                    "kind) VALUES (?, ?, ?, 'pending', ?, 'package')",
                    (package.package_id, digest, payload, now()),
                )
                publish_batch(
                    connection,
                    self.database.data_dir,
                    package,
                    decoded,
                    (package.package_id, result_hash(references)),
                )
        return self.status(package.package_id)

    def status(self, package_id: str) -> dict[str, object]:
        with self.database.read() as connection:
            row = import_row(connection, package_id)
        if row is None:
            raise KeyError(package_id)
        return public_status(row)
