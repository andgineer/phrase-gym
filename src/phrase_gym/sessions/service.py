import json
from collections.abc import Callable
from datetime import datetime, timedelta
from typing import cast
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import AttemptEvidence, EventReceipt, PracticeInput
from phrase_gym.contracts.sessions import (
    PracticeSession,
    SessionManifest,
    SessionStart,
    SessionStatus,
    SessionTransition,
)
from phrase_gym.errors import DomainError
from phrase_gym.practice.support import support_times
from phrase_gym.practice.tables import AttemptRow, PracticeEvent
from phrase_gym.serialization import canonical_json
from phrase_gym.sessions.planning import SessionPlanner
from phrase_gym.sessions.queue import initial_queue
from phrase_gym.storage import Database
from phrase_gym.tables import SessionRow


class SessionService:
    def __init__(self, database: Database, planner: SessionPlanner, clock: Callable[[], datetime]):
        self.db, self.planner, self.clock = database, planner, clock

    @staticmethod
    def model(row: SessionRow, transaction: Session) -> PracticeSession:
        manifest = SessionManifest.model_validate_json(row.manifest)
        earliest = datetime.fromisoformat(row.created_at) - timedelta(minutes=5)
        return PracticeSession(
            id=UUID(row.id),
            status=cast(SessionStatus, row.status),
            version=row.version,
            created_at=datetime.fromisoformat(row.created_at),
            updated_at=datetime.fromisoformat(row.updated_at),
            manifest=manifest,
            cursor=row.cursor or 0,
            last_sequence=row.last_sequence or 0,
            queue=json.loads(row.queue or "[]"),
            attempts=[
                AttemptEvidence.model_validate_json(attempt.payload)
                for attempt in transaction.scalars(
                    select(AttemptRow)
                    .where(
                        AttemptRow.session_id == row.id,
                    )
                    .order_by(AttemptRow.step),
                )
            ],
            dialogue_exposures=[
                EventReceipt.model_validate_json(event.receipt).occurred_at
                for event in transaction.scalars(
                    select(PracticeEvent)
                    .where(PracticeEvent.session_id == row.id)
                    .order_by(PracticeEvent.sequence),
                )
                if PracticeInput.model_validate_json(event.payload).kind == "dialogue_exposure"
            ],
            support_exposures={
                target.phrase.id: [
                    time
                    for time in support_times(transaction, str(target.phrase.id))
                    if time >= earliest
                ]
                for target in manifest.targets
            },
        )

    @staticmethod
    def open_row(transaction: Session) -> SessionRow | None:
        return transaction.scalar(
            select(SessionRow).where(SessionRow.status.in_(("active", "paused"))),
        )

    def current(self) -> PracticeSession | None:
        with self.db.read() as transaction:
            row = self.open_row(transaction)
            return self.model(row, transaction) if row else None

    def get(self, session_id: UUID) -> PracticeSession:
        with self.db.read() as transaction:
            row = transaction.get(SessionRow, str(session_id))
            if row is None:
                raise DomainError("Session not found", 404)
            return self.model(row, transaction)

    def start(self, payload: SessionStart) -> PracticeSession:
        serialized = canonical_json(payload.model_dump_json())
        with self.db.write() as transaction:
            existing = transaction.get(SessionRow, str(payload.id))
            if existing:
                if existing.start_payload != serialized:
                    raise DomainError("Session ID already has different start parameters", 409)
                return self.model(existing, transaction)
            current = self.open_row(transaction)
            if (str(payload.takeover_session_id) if payload.takeover_session_id else None) != (
                current.id if current else None
            ):
                raise DomainError(
                    "Resume the open session or explicitly take over its current ID",
                    409,
                )
            now = self.clock()
            manifest = self.planner.plan(
                transaction,
                payload.language,
                now,
                payload.lesson_id,
                payload.dialogue_id,
            )
            if not manifest.targets:
                raise DomainError(
                    "No due/new targets with ready reference audio and reviewed cues",
                    409,
                )
            queue = [step.model_dump() for step in initial_queue(manifest)]
            if current:
                current.status = "superseded"
                current.open_slot = None
                current.version += 1
                current.updated_at = now.isoformat()
                transaction.flush()
            row = SessionRow(
                id=str(payload.id),
                start_payload=serialized,
                manifest=manifest.model_dump_json(),
                status="active",
                open_slot=1,
                version=1,
                created_at=now.isoformat(),
                updated_at=now.isoformat(),
                queue=json.dumps(queue),
                repairs="[]",
                cursor=0,
                last_sequence=0,
            )
            transaction.add(row)
            return self.model(row, transaction)

    def transition(self, session_id: UUID, payload: SessionTransition) -> PracticeSession:
        with self.db.write() as transaction:
            row = transaction.get(SessionRow, str(session_id))
            if row is None:
                raise DomainError("Session not found", 404)
            if row.status in ("superseded", "completed"):
                raise DomainError("A closed session cannot resume", 409)
            if row.status == payload.status and row.version == payload.expected_version + 1:
                return self.model(row, transaction)
            if row.version != payload.expected_version:
                raise DomainError("Session changed; reload its current version", 409)
            if row.status != payload.status:
                row.status = payload.status
                row.version += 1
                row.updated_at = self.clock().isoformat()
            return self.model(row, transaction)
