import json
from dataclasses import dataclass
from datetime import datetime, timedelta

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import AttemptEvidence
from phrase_gym.contracts.sessions import SessionStep
from phrase_gym.practice.tables import AttemptRow
from phrase_gym.tables import SessionRow

MAX_REPAIRS = 2
INTERVENING_TARGETS = 2


@dataclass
class Advancement:
    occurred_at: datetime
    defer_repairs: bool = False


def advance(
    transaction: Session,
    session: SessionRow,
    evidence: AttemptEvidence,
    queue: list[SessionStep],
    timing: Advancement,
) -> None:
    now, defer_repairs = timing.occurred_at, timing.defer_repairs
    if session.status in ("completed", "superseded"):
        return
    pending = json.loads(session.repairs)
    step = queue[session.cursor]
    extra_count = sum(q.repair and q.target_index == step.target_index for q in queue)
    if (
        evidence.eligibility_reason
        in ("eligible", "repair", "supported", "supported_learning_check")
        and (evidence.rating == "not_yet" or (evidence.original and evidence.original.supported))
        and extra_count < MAX_REPAIRS
        and not defer_repairs
    ):
        pending.append(
            {
                "target_index": step.target_index,
                "after_step": session.cursor,
                "target_id": str(evidence.target_id),
                "eligible_at": (now + timedelta(minutes=5)).isoformat(),
            },
        )
    session.cursor += 1
    evidence_rows = transaction.scalars(
        select(AttemptRow).where(AttemptRow.session_id == session.id),
    ).all()
    attempts = [AttemptEvidence.model_validate_json(r.payload) for r in evidence_rows]
    retained = []
    for repair in pending:
        if defer_repairs:
            retained.append(repair)
            continue
        others = {
            a.target_id
            for a in attempts
            if a.step > repair["after_step"]
            and a.finalized_at
            and str(a.target_id) != repair["target_id"]
        }
        extra = sum(q.repair and q.target_index == repair["target_index"] for q in queue)
        if extra >= MAX_REPAIRS:
            continue
        if (
            now >= datetime.fromisoformat(repair["eligible_at"])
            and len(others) >= INTERVENING_TARGETS
        ):
            queue.append(SessionStep(target_index=repair["target_index"], repair=True))
        else:
            retained.append(repair)
    session.queue = json.dumps([step.model_dump() for step in queue])
    session.repairs = json.dumps(retained)
    if session.cursor >= len(queue):
        session.status = "completed"
        session.open_slot = None
    session.version += 1
    session.updated_at = now.isoformat()
