"""Receipt and preparation enqueue share one transaction, preserving export identity."""

from uuid import UUID

from phrase_gym.contracts.handoff import HandoffEnvelope, HandoffReceipt
from phrase_gym.errors import DomainError
from phrase_gym.preparation.jobs import PreparationJobs, digest
from phrase_gym.preparation.tables import HandoffRow


class HandoffReceiver:
    def __init__(self, jobs: PreparationJobs, settings: dict):
        self.jobs = jobs
        self.settings = settings

    def receive(self, envelope: HandoffEnvelope) -> HandoffReceipt:
        payload = envelope.model_dump_json()
        source = envelope.package.model_copy(deep=True)
        for phrase in source.phrases:
            if phrase.meaning_key != envelope.selected_sense_id:
                raise DomainError(
                    "Handoff phrase meaning key must match the selected sense ID",
                    422,
                )
            if phrase.selected_meaning not in (None, envelope.selected_sense):
                raise DomainError("Handoff phrase contradicts the explicitly selected meaning", 422)
            phrase.selected_meaning = envelope.selected_sense
        with self.jobs.db.write() as transaction:
            existing = transaction.get(HandoffRow, str(envelope.export_id))
            if existing:
                if existing.payload_hash != digest(payload):
                    raise DomainError("Export ID already received with different selected content")
                job = self.jobs.row(transaction, UUID(existing.preparation_id))
            else:
                job = self.jobs.enqueue_in(transaction, source, self.settings)
                transaction.add(
                    HandoffRow(
                        export_id=str(envelope.export_id),
                        payload_hash=digest(payload),
                        payload=payload,
                        preparation_id=job.id,
                    ),
                )
            return self.receipt(envelope.export_id, UUID(job.id), job.status)

    def status(self, export_id: UUID) -> HandoffReceipt:
        with self.jobs.db.read() as transaction:
            row = transaction.get(HandoffRow, str(export_id))
            if row is None:
                raise DomainError("Handoff not found", 404)
            job = self.jobs.row(transaction, UUID(row.preparation_id))
            return self.receipt(export_id, UUID(job.id), job.status)

    @staticmethod
    def receipt(export_id: UUID, job_id: UUID, status: str) -> HandoffReceipt:
        state = {
            "queued": "received",
            "leased": "preparing",
            "awaiting_review": "preparing",
            "failed": "failed",
            "ready": "ready",
        }[status]
        return HandoffReceipt.model_validate(
            {"export_id": export_id, "preparation_id": job_id, "status": state},
        )
