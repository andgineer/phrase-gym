"""Dry-run source edits and objective conflicts before provider work."""

from sqlalchemy.orm import Session

from phrase_gym.contracts.content import TargetInput
from phrase_gym.contracts.preparation import SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.tables import SourceItem, Target, TargetRevision


def source_changes(transaction: Session, source: SourcePackage) -> dict:
    changes = []
    errors = []
    for phrase in source.phrases:
        item = transaction.get(SourceItem, (source.namespace, phrase.source_id))
        target = transaction.get(Target, item.target_id) if item else None
        action = "create"
        if target:
            if (target.language, target.kind, target.meaning_key, target.recall_expression) != (
                source.language,
                phrase.kind,
                phrase.meaning_key,
                phrase.recall_expression,
            ):
                errors.append(
                    f"{phrase.source_id}: changed learning objective requires a new source ID",
                )
            revision = transaction.get(TargetRevision, (target.id, target.current_revision))
            assert revision is not None
            previous = TargetInput.model_validate_json(revision.payload)
            unchanged = previous.text == phrase.text and previous.purpose == phrase.purpose
            if phrase.reference_audio_id is not None:
                unchanged = unchanged and previous.reference_audio_id == phrase.reference_audio_id
            action = "unchanged" if unchanged else "update"
        changes.append(
            {
                "source_id": phrase.source_id,
                "action": action,
                "target_id": target.id if target else None,
                "base_revision": target.current_revision if target else None,
            },
        )
    return {"changes": changes, "identity_errors": errors}


def check_source_objectives(transaction: Session, source: SourcePackage) -> None:
    errors = source_changes(transaction, source)["identity_errors"]
    if errors:
        raise DomainError("; ".join(errors), 422)
