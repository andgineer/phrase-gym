"""Short transactions own leases; provider calls run outside database locks."""

import hashlib
import json
from collections.abc import Callable
from datetime import datetime, timedelta
from uuid import UUID, uuid4

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.clock import utc_now
from phrase_gym.content.source_identity import check_source_objectives
from phrase_gym.contracts.preparation import PreparationJob, SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.preparation import versions
from phrase_gym.preparation.bindings import source_bindings
from phrase_gym.preparation.tables import PreparationRow
from phrase_gym.serialization import canonical_json
from phrase_gym.storage import Database

MAX_ATTEMPTS = 3
LEASE_SECONDS = 600


def digest(value: str) -> str:
    return hashlib.sha256(canonical_json(value).encode()).hexdigest()


class PreparationJobs:
    def __init__(self, database: Database, clock: Callable[[], datetime] = utc_now):
        self.db = database
        self.clock = clock

    def enqueue(self, source: SourcePackage, settings_identity: dict) -> PreparationJob:
        with self.db.write() as transaction:
            row = self.enqueue_in(transaction, source, settings_identity)
            return self.model(row)

    def enqueue_in(self, transaction: Session, source: SourcePackage, settings_identity: dict):
        check_source_objectives(transaction, source)
        payload = json.dumps(
            {
                "source": source.model_dump(mode="json"),
                "settings": settings_identity,
                "prompt_version": versions.PROMPT_VERSION,
                "review_version": versions.REVIEW_VERSION,
                "speech_version": versions.SPEECH_VERSION,
            },
            sort_keys=True,
        )
        input_hash = digest(payload)
        existing = transaction.scalar(
            select(PreparationRow).where(PreparationRow.input_hash == input_hash),
        )
        if existing:
            return existing
        row = PreparationRow(
            id=str(uuid4()),
            input_hash=input_hash,
            payload=payload,
            base_bindings=source_bindings(transaction, source),
            status="queued",
            attempts=0,
            lease_token=None,
            lease_until=None,
            retry_at=self.clock().isoformat(),
            error=None,
            result=None,
            review=None,
            published_import_id=None,
        )
        transaction.add(row)
        transaction.flush()
        return row

    def get(self, job_id: UUID) -> PreparationJob:
        with self.db.read() as transaction:
            return self.model(self.row(transaction, job_id))

    def details(self, job_id: UUID) -> dict:
        with self.db.read() as transaction:
            row = self.row(transaction, job_id)
            return {
                **self.model(row).model_dump(mode="json"),
                "input": json.loads(row.payload),
                "human_review": json.loads(row.review) if row.review else None,
            }

    @staticmethod
    def row(transaction: Session, job_id: UUID) -> PreparationRow:
        row = transaction.get(PreparationRow, str(job_id))
        if row is None:
            raise DomainError("Preparation job not found", 404)
        return row

    @staticmethod
    def model(row: PreparationRow) -> PreparationJob:
        return PreparationJob.model_validate(
            {
                "id": row.id,
                "status": row.status,
                "input_hash": row.input_hash,
                "attempts": row.attempts,
                "error": row.error,
                "result": json.loads(row.result) if row.result else None,
                "published_import_id": row.published_import_id,
            },
        )

    def claim(self, job_id: UUID | None = None) -> tuple[UUID, str, dict, dict] | None:
        now = self.clock()
        with self.db.write() as transaction:
            query = select(PreparationRow).order_by(PreparationRow.retry_at)
            if job_id is not None:
                query = query.where(PreparationRow.id == str(job_id))
            rows = transaction.scalars(query).all()
            for row in rows:
                expired = (
                    row.status == "leased"
                    and row.lease_until
                    and row.lease_until <= now.isoformat()
                )
                eligible = row.status in ("queued", "failed") and row.retry_at <= now.isoformat()
                if not expired and not eligible:
                    continue
                if row.attempts >= MAX_ATTEMPTS:
                    row.status = "failed"
                    row.error = "retry_limit: explicit retry required"
                    row.lease_token = row.lease_until = None
                    continue
                row.status = "leased"
                row.attempts += 1
                token = str(uuid4())
                row.lease_token = token
                row.lease_until = (now + timedelta(seconds=LEASE_SECONDS)).isoformat()
                return (
                    UUID(row.id),
                    token,
                    json.loads(row.payload),
                    json.loads(row.result or "{}"),
                )
        return None

    def checkpoint(self, job_id: UUID, token: str, result: dict, *, complete: bool = False) -> None:
        with self.db.write() as transaction:
            row = self.row(transaction, job_id)
            self.check_lease(row, token)
            row.result = json.dumps(result, ensure_ascii=False)
            row.error = None
            row.lease_until = (self.clock() + timedelta(seconds=LEASE_SECONDS)).isoformat()
            if complete:
                row.status = "awaiting_review"
                row.lease_token = row.lease_until = None

    def fail(self, job_id: UUID, token: str, code: str) -> None:
        with self.db.write() as transaction:
            row = self.row(transaction, job_id)
            self.check_lease(row, token)
            row.status = "failed"
            row.error = code
            row.retry_at = (self.clock() + timedelta(seconds=30 * row.attempts)).isoformat()
            row.lease_token = row.lease_until = None

    def retry(self, job_id: UUID, *, regenerate: bool = False) -> PreparationJob:
        with self.db.write() as transaction:
            row = self.row(transaction, job_id)
            if row.status != "failed":
                raise DomainError("Only failed preparation can be retried")
            if regenerate and row.result:
                previous = json.loads(row.result)
                archive = previous.pop("previous_runs", [])
                archive.append(previous)
                row.result = json.dumps({"previous_runs": archive}, ensure_ascii=False)
            row.status = "queued"
            row.attempts = 0
            row.retry_at = self.clock().isoformat()
            return self.model(row)

    def check_lease(self, row: PreparationRow, token: str) -> None:
        if (
            row.status != "leased"
            or row.lease_token != token
            or not row.lease_until
            or row.lease_until <= self.clock().isoformat()
        ):
            raise DomainError("Preparation lease expired or was replaced")
