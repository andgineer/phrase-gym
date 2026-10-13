import json
from datetime import datetime, timedelta
from uuid import UUID, uuid5

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import EventReceipt, PracticeInput
from phrase_gym.practice.tables import AttemptRow, PracticeEvent
from phrase_gym.tables import ExposureRow


def support_times(transaction: Session, target_id: str) -> list[datetime]:
    exposures = transaction.scalars(
        select(ExposureRow).where(ExposureRow.target_id == target_id),
    ).all()
    attempts = transaction.scalars(
        select(AttemptRow).where(AttemptRow.target_id == target_id),
    ).all()
    dialogues = {
        row.id: json.loads(row.payload)["session_id"]
        for row in exposures
        if json.loads(row.payload).get("kind") == "dialogue"
    }
    times = [
        datetime.fromisoformat(row.received_at) for row in exposures if row.id not in dialogues
    ]
    attempt_ids = {UUID(row.id) for row in attempts}
    events = transaction.scalars(
        select(PracticeEvent).where(
            PracticeEvent.session_id.in_(
                {row.session_id for row in attempts} | set(dialogues.values()),
            ),
        ),
    )
    for row in events:
        event = PracticeInput.model_validate_json(row.payload)
        dialogue = (
            event.kind == "dialogue_exposure" and str(uuid5(event.id, target_id)) in dialogues
        )
        if not dialogue and event.kind not in (
            "answer",
            "explain",
            "reference",
            "capture_complete",
        ):
            continue
        receipt = EventReceipt.model_validate_json(row.receipt)
        if dialogue or receipt.attempt_id in attempt_ids:
            times.append(receipt.occurred_at)
    return sorted(set(times))


def recent_support(transaction: Session, target_id: str, now: datetime) -> bool:
    return any(
        now - timedelta(minutes=5) <= time <= now for time in support_times(transaction, target_id)
    )
