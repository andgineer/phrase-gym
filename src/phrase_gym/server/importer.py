import asyncio
import hashlib
import json
import shutil
import sqlite3
import tempfile
from datetime import UTC, datetime
from pathlib import Path
from typing import Protocol
from uuid import NAMESPACE_URL, uuid5

from phrase_gym.server.contracts import ImportCard, PhrasePackage
from phrase_gym.server.storage import Database, write_private


def now() -> str:
    return datetime.now(UTC).isoformat()


def canonical(package: PhrasePackage) -> str:
    return package.model_dump_json(exclude_none=True)


def card_identity(source_id: str, item_id: str) -> str:
    encoded = json.dumps([source_id, item_id], separators=(",", ":"))
    return str(uuid5(NAMESPACE_URL, encoded))


class ImportConflictError(ValueError):
    pass


class Speech(Protocol):
    async def synthesize(self, text: str) -> bytes: ...


class Importer:
    def __init__(self, database: Database, speech: Speech):
        self.database = database
        self.speech = speech
        self.lock = asyncio.Lock()
        self.tasks: set[asyncio.Task[None]] = set()

    def submit(self, package: PhrasePackage) -> dict[str, object]:
        payload = canonical(package)
        digest = hashlib.sha256(payload.encode()).hexdigest()
        with self.database.write() as connection:
            previous = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package.package_id,),
            ).fetchone()
            if previous:
                if previous["content_hash"] != digest:
                    raise ImportConflictError(
                        "This package_id was already used for different content; "
                        "use a new package_id.",
                    )
                return self._public(previous)
            busy = connection.execute(
                "SELECT package_id FROM imports WHERE status='preparing' LIMIT 1",
            ).fetchone()
            if busy:
                raise ImportConflictError(
                    f"Package {busy['package_id']} is preparing; wait for it to finish "
                    "before importing another.",
                )
            placeholders = ",".join("?" for _ in package.cards)
            # Only parameter placeholders enter the SQL text; all values remain bound.
            conflicts = connection.execute(
                f"SELECT source_id, item_id FROM cards "  # noqa: S608
                f"WHERE source_id=? AND item_id IN ({placeholders})",
                (package.source_id, *(card.item_id for card in package.cards)),
            ).fetchall()
            if conflicts:
                items = ", ".join(row["item_id"] for row in conflicts)
                raise ImportConflictError(
                    f"Source items already exist ({items}); no cards were changed. "
                    "Replacement is available in a later stage.",
                )
            connection.execute(
                "INSERT INTO imports VALUES (?, ?, ?, 'preparing', NULL, NULL, ?)",
                (package.package_id, digest, payload, now()),
            )
        self._start(package)
        return self.status(package.package_id)

    def retry(self, package_id: str) -> dict[str, object]:
        with self.database.write() as connection:
            row = connection.execute(
                "SELECT * FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
            if not row:
                raise KeyError(package_id)
            if row["status"] == "added":
                return self._public(row)
            if row["status"] == "preparing":
                return self._public(row)
            busy = connection.execute(
                "SELECT package_id FROM imports WHERE status='preparing' AND package_id<>? LIMIT 1",
                (package_id,),
            ).fetchone()
            if busy:
                raise ImportConflictError(
                    f"Package {busy['package_id']} is preparing; wait for it to finish "
                    "before retrying.",
                )
            connection.execute(
                "UPDATE imports SET status='preparing', error=NULL WHERE package_id=?",
                (package_id,),
            )
            package = PhrasePackage.model_validate_json(row["payload"])
        self._start(package)
        return self.status(package_id)

    def _start(self, package: PhrasePackage) -> None:
        task = asyncio.create_task(self._prepare(package))
        self.tasks.add(task)
        task.add_done_callback(self.tasks.discard)

    async def _prepare(self, package: PhrasePackage) -> None:
        async with self.lock:
            staging = Path(tempfile.mkdtemp(prefix="phrase-gym-import-"))
            created: list[Path] = []
            try:
                prepared: list[tuple[ImportCard, Path]] = []
                for card in package.cards:
                    audio = await self.speech.synthesize(card.answer)
                    path = staging / f"{card_identity(package.source_id, card.item_id)}.wav"
                    path.write_bytes(audio)
                    prepared.append((card, path))
                references = self.database.data_dir / "references"
                references.mkdir(mode=0o700, exist_ok=True)
                references.chmod(0o700)
                with self.database.write() as connection:
                    placeholders = ",".join("?" for _ in package.cards)
                    # Only parameter placeholders enter the SQL text; all values remain bound.
                    repeated = connection.execute(
                        f"SELECT item_id FROM cards WHERE source_id=? "  # noqa: S608
                        f"AND item_id IN ({placeholders})",
                        (package.source_id, *(card.item_id for card in package.cards)),
                    ).fetchall()
                    if repeated:
                        raise ImportConflictError(
                            "A source item appeared while audio was preparing; "
                            "no cards were added.",
                        )
                    for card, staged_path in prepared:
                        card_id = card_identity(package.source_id, card.item_id)
                        destination = references / f"{card_id}.wav"
                        if destination.exists():
                            owner = connection.execute(
                                "SELECT id FROM cards WHERE reference_path=?",
                                (str(destination),),
                            ).fetchone()
                            if owner:
                                raise ImportConflictError(
                                    "Reference audio is already owned by an existing card.",
                                )
                            destination.unlink()
                        write_private(destination, staged_path.read_bytes())
                        created.append(destination)
                        connection.execute(
                            "INSERT INTO cards VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
                            (
                                card_id,
                                package.source_id,
                                card.item_id,
                                package.source_label,
                                str(package.source_link) if package.source_link else None,
                                card.answer,
                                card.explanation,
                                json.dumps([cue.model_dump() for cue in card.cues]),
                                json.dumps(card.accepted_variants),
                                card.detailed_explanation,
                                str(destination),
                                now(),
                            ),
                        )
                    connection.execute(
                        "UPDATE imports SET status='added', added_count=?, error=NULL "
                        "WHERE package_id=?",
                        (len(prepared), package.package_id),
                    )
            # Background failures must clean up files and remain visible for retry.
            except Exception as error:  # noqa: BLE001
                for path in created:
                    path.unlink(missing_ok=True)
                with self.database.write() as connection:
                    connection.execute(
                        "UPDATE imports SET status='error', error=? WHERE package_id=?",
                        (str(error), package.package_id),
                    )
            finally:
                shutil.rmtree(staging, ignore_errors=True)

    def status(self, package_id: str) -> dict[str, object]:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT package_id, status, error, added_count FROM imports WHERE package_id=?",
                (package_id,),
            ).fetchone()
        if not row:
            raise KeyError(package_id)
        return dict(row)

    @staticmethod
    def _public(row: sqlite3.Row) -> dict[str, object]:
        return {
            "package_id": row["package_id"],
            "status": row["status"],
            "error": row["error"],
            "added_count": row["added_count"],
        }
