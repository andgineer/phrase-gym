"""Operator corrections retain model evidence and invalidate the previous review hash."""

import json
from uuid import UUID

from phrase_gym.contracts.content import DialogueInput
from phrase_gym.contracts.preparation import PreparationDraft, SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.preparation.chunks import reconcile_draft
from phrase_gym.preparation.jobs import PreparationJobs, digest
from phrase_gym.preparation.validation import assemble


def revise_draft(
    jobs: PreparationJobs,
    job_id: UUID,
    draft: PreparationDraft,
    reviewer: str,
    notes: str,
):
    with jobs.db.write() as transaction:
        row = jobs.row(transaction, job_id)
        if row.status != "awaiting_review":
            raise DomainError("Only a completed unpublished draft can be corrected")
        result = json.loads(row.result or "{}")
        source = SourcePackage.model_validate(json.loads(row.payload)["source"])
        assemble(source, draft)
        draft, mapping = reconcile_draft(transaction, source, draft)
        corrected = assemble(source, draft)
        previous = result["package"]
        for target in corrected.targets:
            original = next(p for p in previous["targets"] if p["source_id"] == target.source_id)
            target.reference_audio_id = UUID(original["reference_audio_id"])
        corrected.dialogues = [DialogueInput.model_validate(d) for d in previous["dialogues"]]
        history = result.setdefault("operator_revisions", [])
        history.append(
            {
                "reviewer": reviewer,
                "notes": notes,
                "previous_draft": result["draft"],
                "previous_result_hash": digest(json.dumps(result)),
            },
        )
        result["draft"] = draft.model_dump(mode="json")
        result["chunk_id_mapping"] = mapping
        result["package"] = corrected.model_dump(mode="json")
        result["advisory_review_applies_to_original_draft_only"] = True
        row.result = json.dumps(result, ensure_ascii=False)
        return jobs.model(row)
