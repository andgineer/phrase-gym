"""Hash-bound operator approval and atomic content/job publication."""

import json
from uuid import UUID, uuid4

from phrase_gym.content.imports import ImportService
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.preparation import HumanReview, PreparationDraft, SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.preparation.bindings import source_bindings
from phrase_gym.preparation.chunks import reconcile_draft
from phrase_gym.preparation.jobs import PreparationJobs, digest
from phrase_gym.tables import ImportRow, SourceItem, Target, TargetRevision


def publish_reviewed(
    jobs: PreparationJobs,
    imports: ImportService,
    job_id: UUID,
    review: HumanReview,
):
    with jobs.db.write() as transaction:
        row = jobs.row(transaction, job_id)
        result = json.loads(row.result or "{}")
        if review.input_hash != row.input_hash or review.draft_hash != digest(json.dumps(result)):
            raise DomainError("Review hashes do not match this exact preparation result", 422)
        if row.status == "ready":
            if row.review != review.model_dump_json():
                raise DomainError("Published preparation already has a different review")
            return jobs.model(row)
        if row.status != "awaiting_review":
            raise DomainError("Only a completed draft can be reviewed for publication")
        source = SourcePackage.model_validate(json.loads(row.payload)["source"])
        if source_bindings(transaction, source) != row.base_bindings:
            raise DomainError("Source content changed during preparation; enqueue a new revision")
        package = ManualPackage.model_validate(result["package"])
        draft = PreparationDraft.model_validate(result["draft"])
        normalized, _mapping = reconcile_draft(transaction, source, draft)
        if normalized != draft:
            raise DomainError("Chunk identities require correction and a new review hash", 422)
        if set(review.accepted_cues) != {p.source_id for p in source.phrases}:
            raise DomainError("Review must explicitly decide every source phrase", 422)
        apply_review(transaction, package, draft, review)
        report = imports.validator.report(transaction, package)
        if report.errors:
            raise DomainError("; ".join(report.errors), 422)
        report.id = uuid4()
        report.status = "staged"
        transaction.add(
            ImportRow(
                id=str(report.id),
                payload=package.model_dump_json(),
                report=report.model_dump_json(),
                status="staged",
                created_at=jobs.clock().isoformat(),
            ),
        )
        transaction.flush()
        imports.publish_in(transaction, report.id)
        row.status = "ready"
        row.review = review.model_dump_json()
        row.published_import_id = str(report.id)
        return jobs.model(row)


def apply_review(transaction, package, draft, review):
    for target in package.targets:
        selected = review.accepted_cues[target.source_id]
        allowed = {
            c.id
            for c in next(p for p in draft.phrases if p.source_id == target.source_id).candidates
        }
        if len(selected) != len(set(selected)) or not set(selected) <= allowed:
            raise DomainError("Review refers to duplicate or unknown cues", 422)
        target.cues = [cue for cue in target.cues if cue.id in selected]
        item = transaction.get(SourceItem, (package.namespace, target.source_id))
        existing = transaction.get(Target, item.target_id) if item else None
        if existing:
            revision = transaction.get(TargetRevision, (existing.id, existing.current_revision))
            assert revision is not None
            previous = ManualPackage.model_validate(
                {
                    **package.model_dump(),
                    "targets": [json.loads(revision.payload)],
                    "lessons": [],
                    "dialogues": [],
                },
            ).targets[0]
            # Preparing another native language must retain reviewed help for earlier languages.
            target.explanations = {**previous.explanations, **target.explanations}
