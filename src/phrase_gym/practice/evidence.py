from datetime import datetime
from uuid import UUID, uuid5

from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import (
    AttemptEvidence,
    EventReceipt,
    OriginalResponse,
    PracticeInput,
)
from phrase_gym.contracts.sessions import PlannedTarget, SessionStep
from phrase_gym.practice.support import recent_support
from phrase_gym.practice.tables import AttemptRow, ChunkHistoryRow
from phrase_gym.tables import SessionRow


def attempt_id(session_id: UUID, step: int) -> UUID:
    return uuid5(session_id, f"step:{step}")


def create_attempt(
    transaction: Session,
    session: SessionRow,
    item: PlannedTarget,
    step: SessionStep,
    now: datetime,
) -> tuple[AttemptRow, AttemptEvidence]:
    supported = recent_support(transaction, str(item.phrase.id), now)
    evidence = AttemptEvidence(
        id=attempt_id(UUID(session.id), session.cursor),
        step=session.cursor,
        target_id=item.phrase.id,
        revision=item.phrase.revision,
        cue_id=item.cue_id,
        review_group_id=item.review_group_id,
        repair=step.repair,
        supported=supported,
        presented_at=now,
    )
    row = AttemptRow(
        id=str(evidence.id),
        session_id=session.id,
        target_id=str(item.phrase.id),
        step=session.cursor,
        payload=evidence.model_dump_json(),
    )
    transaction.add(row)
    for chunk in item.hidden_chunks:
        history = transaction.get(ChunkHistoryRow, (str(item.phrase.id), chunk))
        if history is None:
            history = ChunkHistoryRow(
                target_id=str(item.phrase.id),
                chunk_id=chunk,
                presentations=0,
                recalls=0,
                failures=0,
                last_presented="",
            )
            transaction.add(history)
        history.presentations += 1
        history.last_presented = now.isoformat()
    return row, evidence


def observe(evidence: AttemptEvidence, event: PracticeInput, now: datetime) -> None:
    if event.kind in ("respond", "answer", "rate") and evidence.original is None:
        evidence.original = OriginalResponse(
            modality=event.modality,
            supported=evidence.supported,
            occurred_at=now,
        )
    if event.kind in ("answer", "explain", "reference"):
        evidence.supported = True
        evidence.revealed = True
    if event.kind == "explain":
        evidence.expanded = True
    if event.kind == "respond":
        evidence.capture_pending = True
    if event.kind == "capture_complete":
        evidence.revealed = True
        evidence.capture_pending = False
    if event.kind == "failure":
        evidence.technical_failure = True
        evidence.capture_pending = False


def refresh_support(
    transaction: Session,
    evidence: AttemptEvidence,
    event: PracticeInput,
    receipt: EventReceipt,
) -> None:
    if evidence.original is None:
        evidence.supported |= recent_support(
            transaction,
            str(evidence.target_id),
            receipt.occurred_at,
        )
    if event.kind == "respond":
        receipt.capture_supported = evidence.supported or evidence.revealed
