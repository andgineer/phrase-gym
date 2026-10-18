import json
from datetime import datetime
from uuid import uuid5

from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import AttemptEvidence, EventReceipt, PracticeInput
from phrase_gym.contracts.sessions import SessionManifest, SessionStep
from phrase_gym.errors import DomainError
from phrase_gym.practice.evidence import create_attempt, observe, refresh_support
from phrase_gym.practice.policy import ReviewPolicy
from phrase_gym.practice.repairs import advance
from phrase_gym.practice.reviews import ReviewContext, finalize_review
from phrase_gym.practice.tables import AttemptRow, CueReportRow
from phrase_gym.tables import ExposureRow, SessionRow


class ObservationProcessor:
    def __init__(self, policy: ReviewPolicy):
        self.policy = policy

    @staticmethod
    def introduce(
        transaction: Session,
        session: SessionRow,
        manifest: SessionManifest,
        event: PracticeInput,
        now: datetime,
    ) -> None:
        if not manifest.dialogue or session.cursor != 0:
            raise DomainError("Only a dialogue introduction exposes its script", 422)
        for target in manifest.targets:
            transaction.add(
                ExposureRow(
                    id=str(uuid5(event.id, str(target.phrase.id))),
                    target_id=str(target.phrase.id),
                    revision=target.phrase.revision,
                    payload=json.dumps({"kind": "dialogue", "session_id": session.id}),
                    received_at=now.isoformat(),
                ),
            )

    def dialogue_event(
        self,
        transaction: Session,
        session: SessionRow,
        event: PracticeInput,
        now: datetime,
    ) -> bool:
        manifest = SessionManifest.model_validate_json(session.manifest)
        queue = [SessionStep.model_validate(step) for step in json.loads(session.queue)]
        if event.kind == "dialogue_exposure":
            self.introduce(transaction, session, manifest, event, now)
            return True
        if event.kind == "dialogue_begin":
            if not manifest.dialogue or session.cursor != 0:
                raise DomainError("Only a prepared dialogue can begin rehearsal", 422)
            queue[0].introduced = True
            session.queue = json.dumps([step.model_dump() for step in queue])
            return True
        return False

    def process(
        self,
        transaction: Session,
        session: SessionRow,
        event: PracticeInput,
        receipt: EventReceipt,
    ) -> None:
        manifest = SessionManifest.model_validate_json(session.manifest)
        queue = [SessionStep.model_validate(step) for step in json.loads(session.queue)]
        if self.dialogue_event(transaction, session, event, receipt.received_at):
            return
        item = manifest.targets[queue[event.step].target_index]
        row = transaction.get(AttemptRow, str(receipt.attempt_id))
        if row is None:
            if event.kind != "present":
                raise DomainError("Present the cue before reporting a response", 409)
            row, evidence = create_attempt(
                transaction,
                session,
                item,
                queue[event.step],
                receipt.occurred_at,
            )
        else:
            evidence = AttemptEvidence.model_validate_json(row.payload)
        refresh_support(transaction, evidence, event, receipt)
        observe(evidence, event, receipt.occurred_at)
        if event.kind == "rate":
            if evidence.rating is not None or event.rating is None:
                raise DomainError("Attempt already finalized", 409)
            evidence.rating = event.rating
            finalize_review(
                transaction,
                ReviewContext(session, item, evidence, receipt, receipt.occurred_at),
                self.policy,
            )
        if event.kind == "report":
            self.report(transaction, item, evidence, event)
        if event.kind == "detour":
            queue[event.step].dialogue_turn = None
            session.queue = json.dumps([step.model_dump() for step in queue])
        row.payload = evidence.model_dump_json()
        transaction.flush()
        if event.kind in ("rate", "report"):
            advance(
                transaction,
                session,
                evidence,
                queue,
                receipt.occurred_at,
            )

    @staticmethod
    def report(transaction: Session, item, evidence: AttemptEvidence, event: PracticeInput) -> None:
        key = (str(item.phrase.id), item.phrase.revision, item.cue_id)
        if transaction.get(CueReportRow, key) is None:
            transaction.add(
                CueReportRow(
                    target_id=key[0],
                    revision=key[1],
                    cue_id=key[2],
                    reason=event.reason or "",
                ),
            )
        evidence.eligibility_reason = "reported_cue"
        evidence.finalized_at = event.occurred_at
