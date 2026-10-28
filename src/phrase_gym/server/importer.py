import hashlib
import json
import sqlite3
from datetime import UTC, datetime

from phrase_gym.server.contracts import PhrasePackage
from phrase_gym.server.publication import publish_batch
from phrase_gym.server.reference_audio import ImportConflictError, decode_references
from phrase_gym.server.storage import Database


def now() -> str:
    return datetime.now(UTC).isoformat()


class Importer:
    def __init__(self, database: Database):
        self.database = database

    def submit(
        self,
        package: PhrasePackage,
        references: dict[str, str] | None = None,
    ) -> dict[str, object]:
        payload = package.model_dump_json(exclude_none=True)
        decoded = decode_references(package, references) if references is not None else None
        digest_data = json.dumps({"package": payload, "references": references}, sort_keys=True)
        digest = hashlib.sha256(digest_data.encode()).hexdigest()
        with self.database.write() as connection:
            previous = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package.package_id,),
            ).fetchone()
            if previous:
                if previous["content_hash"] != digest:
                    raise ImportConflictError(
                        "This package_id has different content; use a new ID.",
                    )
                return self._public(previous)
            self._reject_existing_items(connection, package)
            connection.execute(
                "INSERT INTO imports (package_id, content_hash, payload, status, created_at) "
                "VALUES (?, ?, ?, 'pending', ?)",
                (package.package_id, digest, payload, now()),
            )
            if decoded is not None and references is not None:
                publish_batch(
                    connection,
                    self.database.data_dir,
                    package,
                    decoded,
                    (package.package_id, self._result_hash(references)),
                )
        return self.status(package.package_id)

    @staticmethod
    def _reject_existing_items(connection: sqlite3.Connection, package: PhrasePackage) -> None:
        for card in package.cards:
            row = connection.execute(
                "SELECT 1 FROM cards WHERE source_id=? AND item_id=?",
                (package.source_id, card.item_id),
            ).fetchone()
            if row:
                raise ImportConflictError(
                    f"cards.{card.item_id}: source item already exists; no cards were changed.",
                )

    def retry(self, package_id: str) -> dict[str, object]:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
            if row is None:
                raise KeyError(package_id)
            if row["status"] in {"error", "preparing"}:
                connection.execute(
                    "UPDATE imports SET status='pending', error=NULL WHERE package_id=?",
                    (package_id,),
                )
        return self.status(package_id)

    def next_work(self) -> dict[str, object] | None:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE status IN ('pending', 'preparing') "
                "ORDER BY created_at LIMIT 1",
            ).fetchone()
            if row is None:
                return None
            connection.execute(
                "UPDATE imports SET status='preparing' WHERE package_id=?",
                (row["package_id"],),
            )
            return {"package_id": row["package_id"], "package": json.loads(row["payload"])}

    def fail(self, package_id: str, message: str) -> dict[str, object]:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
            if row is None:
                raise KeyError(package_id)
            if row["status"] != "added":
                connection.execute(
                    "UPDATE imports SET status='error', error=? WHERE package_id=?",
                    (message, package_id),
                )
        return self.status(package_id)

    def publish(self, package_id: str, result_id: str, audio: dict[str, str]) -> dict[str, object]:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
            if row is None:
                raise KeyError(package_id)
            result_hash = self._result_hash(audio)
            if row["status"] == "added":
                if row["result_id"] != result_id or row["result_hash"] != result_hash:
                    raise ImportConflictError(
                        "A different result was already published for this package.",
                    )
                return self._public(row)
            if row["status"] not in {"pending", "preparing"}:
                raise ImportConflictError(
                    "This package is not pending preparation.",
                )
            package = PhrasePackage.model_validate_json(row["payload"])
            decoded = decode_references(package, audio)
            self._reject_existing_items(connection, package)
            publish_batch(
                connection,
                self.database.data_dir,
                package,
                decoded,
                (result_id, result_hash),
            )
        return self.status(package_id)

    @staticmethod
    def _result_hash(audio: dict[str, str]) -> str:
        return hashlib.sha256(json.dumps(audio, sort_keys=True).encode()).hexdigest()

    def status(self, package_id: str) -> dict[str, object]:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
        if row is None:
            raise KeyError(package_id)
        return self._public(row)

    def list_statuses(self) -> list[dict[str, object]]:
        with self.database.read() as connection:
            rows = connection.execute(
                "SELECT * FROM imports ORDER BY created_at DESC",
            ).fetchall()
        return [self._public(row) for row in rows]

    @staticmethod
    def _public(row: sqlite3.Row) -> dict[str, object]:
        return {
            "package_id": row["package_id"],
            "status": row["status"],
            "error": row["error"],
            "added_count": row["added_count"],
        }
