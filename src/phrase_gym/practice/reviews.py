import json
from dataclasses import dataclass
from datetime import datetime, timedelta

from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import AttemptEvidence, EventReceipt
from phrase_gym.contracts.sessions import PlannedTarget, SessionManifest
from phrase_gym.practice.policy import POLICY_VERSION, ReviewPolicy
from phrase_gym.practice.tables import ChunkHistoryRow, CueReportRow, ReviewReceiptRow
from phrase_gym.tables import ScheduleRow, SessionRow, Target


@dataclass
class ReviewContext:
    session: SessionRow
    item: PlannedTarget
    evidence: AttemptEvidence
    receipt: EventReceipt
    now: datetime


def review_reason(transaction: Session, context: ReviewContext) -> str | None:
    session, item, evidence = context.session, context.item, context.evidence
    target = transaction.get(Target, str(item.phrase.id))
    schedule = transaction.get(ScheduleRow, str(item.phrase.id))
    reasons = [
        (
            SessionManifest.model_validate_json(session.manifest).policy_version != POLICY_VERSION,
            "stale_policy",
        ),
        (session.status in ("superseded", "completed"), "closed_session"),
        (target is None or target.current_revision != item.phrase.revision, "stale_content"),
        (
            transaction.get(CueReportRow, (str(item.phrase.id), item.phrase.revision, item.cue_id))
            is not None,
            "reported_cue",
        ),
        (evidence.technical_failure or evidence.capture_pending, "technical_failure"),
        (evidence.original is None or evidence.original.supported, "supported"),
        (evidence.repair, "repair"),
        (
            transaction.get(ReviewReceiptRow, str(item.review_group_id)) is not None,
            "review_group_already_finalized",
        ),
        ((schedule.version if schedule else 0) != item.schedule_version, "stale_schedule"),
        (
            schedule is not None and datetime.fromisoformat(schedule.due_at) > context.now,
            "early_practice",
        ),
    ]
    return next((reason for condition, reason in reasons if condition), None)


def update_coverage(transaction: Session, context: ReviewContext, reason: str | None) -> None:
    evidence = context.evidence
    if (
        not evidence.original
        or evidence.original.supported
        or evidence.technical_failure
        or reason not in (None, "repair")
    ):
        return
    for chunk in context.item.hidden_chunks:
        history = transaction.get(ChunkHistoryRow, (str(context.item.phrase.id), chunk))
        if history:
            history.recalls += int(evidence.rating != "not_yet")
            history.failures += int(evidence.rating == "not_yet")


def finalize_review(transaction: Session, context: ReviewContext, policy: ReviewPolicy) -> None:
    item, evidence, receipt, now = context.item, context.evidence, context.receipt, context.now
    if evidence.rating is None:
        raise ValueError("Finalization requires a rating")
    reason = review_reason(transaction, context)
    evidence.finalized_at = now
    evidence.eligibility_reason = reason or "eligible"
    receipt.eligibility_reason = evidence.eligibility_reason
    schedule = transaction.get(ScheduleRow, str(item.phrase.id))
    group = transaction.get(ReviewReceiptRow, str(item.review_group_id))
    before = schedule.memory_state if schedule else None
    if reason is None:
        memory, due = policy.review(before, evidence.rating, now)
        if schedule is None:
            schedule = ScheduleRow(target_id=str(item.phrase.id), version=0, due_at=due.isoformat())
            transaction.add(schedule)
        schedule.version += 1
        schedule.due_at = due.isoformat()
        schedule.memory_state = memory
        schedule.policy_version = POLICY_VERSION
        receipt.schedule_changed = True
    elif reason == "supported" and schedule is None and group is None:
        schedule = ScheduleRow(
            target_id=str(item.phrase.id),
            version=1,
            due_at=(now + timedelta(days=1)).isoformat(),
            memory_state=None,
            policy_version=POLICY_VERSION,
        )
        transaction.add(schedule)
        receipt.schedule_changed = True
        receipt.eligibility_reason = "supported_learning_check"
        evidence.eligibility_reason = receipt.eligibility_reason
    if schedule:
        receipt.schedule_version = schedule.version
        receipt.due_at = datetime.fromisoformat(schedule.due_at)
    if group is None and reason in (None, "supported"):
        transaction.add(
            ReviewReceiptRow(
                group_id=str(item.review_group_id),
                target_id=str(item.phrase.id),
                attempt_id=str(evidence.id),
                payload=json.dumps(
                    {
                        "receipt": receipt.model_dump(mode="json"),
                        "policy_version": POLICY_VERSION,
                        "parameters": json.loads(policy.parameters()),
                        "before": json.loads(before) if before else None,
                        "after": json.loads(schedule.memory_state)
                        if schedule and schedule.memory_state
                        else None,
                    },
                ),
            ),
        )
    update_coverage(transaction, context, reason)
