import json
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import EventBatch, EventReceipt, PracticeInput
from phrase_gym.contracts.sessions import PracticeBatchReceipt
from phrase_gym.errors import DomainError
from phrase_gym.practice.evidence import attempt_id
from phrase_gym.practice.observations import ObservationProcessor
from phrase_gym.practice.policy import ReviewPolicy
from phrase_gym.practice.tables import PracticeEvent
from phrase_gym.serialization import canonical_json
from phrase_gym.sessions.service import SessionService
from phrase_gym.storage import Database
from phrase_gym.tables import SessionRow


class PracticeService:
    def __init__(
        self,
        database: Database,
        sessions: SessionService,
        policy: ReviewPolicy,
        clock: Callable[[], datetime],
    ):
        self.db, self.sessions, self.clock = database, sessions, clock
        self.processor = ObservationProcessor(policy)

    def receipts(self, session_id: UUID) -> list[EventReceipt]:
        self.sessions.get(session_id)
        with self.db.read() as transaction:
            return [
                EventReceipt.model_validate_json(row.receipt)
                for row in transaction.scalars(
                    select(PracticeEvent)
                    .where(PracticeEvent.session_id == str(session_id))
                    .order_by(PracticeEvent.sequence),
                )
            ]

    def submit(self, session_id: UUID, batch: EventBatch) -> PracticeBatchReceipt:
        with self.db.write() as transaction:
            session = transaction.get(SessionRow, str(session_id))
            if session is None:
                raise DomainError("Session not found", 404)
            receipts = [self.accept(transaction, session, event) for event in batch.events]
            state = self.sessions.model(session, transaction)
        return PracticeBatchReceipt(receipts=receipts, session=state)

    def accept(
        self,
        transaction: Session,
        session: SessionRow,
        event: PracticeInput,
    ) -> EventReceipt:
        payload = canonical_json(event.model_dump_json())
        existing = transaction.get(PracticeEvent, str(event.id))
        if existing:
            if existing.payload != payload or existing.session_id != session.id:
                raise DomainError("Event ID already has a different payload or session", 409)
            return EventReceipt.model_validate_json(existing.receipt)
        if event.sequence != session.last_sequence + 1:
            raise DomainError(f"Expected event sequence {session.last_sequence + 1}", 409)
        if event.step != session.cursor or event.step >= len(json.loads(session.queue)):
            raise DomainError("Event does not address the current session step", 409)
        receipt = self.event_receipt(transaction, session, event)
        self.processor.process(transaction, session, event, receipt)
        session.last_sequence = event.sequence
        transaction.add(
            PracticeEvent(
                id=str(event.id),
                session_id=session.id,
                sequence=event.sequence,
                payload=payload,
                receipt=receipt.model_dump_json(),
            ),
        )
        transaction.flush()
        return receipt

    def event_receipt(
        self,
        transaction: Session,
        session: SessionRow,
        event: PracticeInput,
    ) -> EventReceipt:
        now = self.clock().astimezone(UTC)
        start = datetime.fromisoformat(session.created_at)
        claimed = event.occurred_at.astimezone(UTC)
        valid = start - timedelta(minutes=5) <= claimed <= now + timedelta(minutes=5)
        occurred = max(start, min(claimed, now)) if valid else now
        prior = transaction.scalar(
            select(PracticeEvent).where(
                PracticeEvent.session_id == session.id,
                PracticeEvent.sequence == session.last_sequence,
            ),
        )
        if prior:
            occurred = max(occurred, EventReceipt.model_validate_json(prior.receipt).occurred_at)
        return EventReceipt(
            id=event.id,
            sequence=event.sequence,
            attempt_id=attempt_id(UUID(session.id), event.step),
            received_at=now,
            occurred_at=occurred,
            clock_adjusted=claimed != occurred,
        )
