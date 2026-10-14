"""Deterministic draft conformance and reviewed publication assembly."""

from phrase_gym.content.cues import has_visible_hint
from phrase_gym.contracts.content import Cue, ManualPackage, TargetInput
from phrase_gym.contracts.preparation import PreparationDraft, SourcePackage
from phrase_gym.errors import DomainError


def assemble(
    source: SourcePackage,
    draft: PreparationDraft,
    accepted: dict[str, list[str]] | None = None,
) -> ManualPackage:
    if {item.source_id for item in draft.phrases} != {p.source_id for p in source.phrases}:
        raise DomainError("Draft source IDs must exactly match the source package", 422)
    if len(draft.phrases) != len(source.phrases):
        raise DomainError("Draft source IDs must be unique", 422)
    targets = []
    for phrase in source.phrases:
        prepared = next(p for p in draft.phrases if p.source_id == phrase.source_id)
        ids = {c.id for c in prepared.chunks}
        if not set(prepared.uncovered_chunks) <= ids:
            raise DomainError("Uncovered chunks must exist", 422)
        if not prepared.candidates and not prepared.no_reliable_cue_reason:
            raise DomainError("No-cue drafts require a reason", 422)
        candidate_ids = {c.id for c in prepared.candidates}
        selection = accepted.get(phrase.source_id, []) if accepted is not None else candidate_ids
        if len(selection) != len(set(selection)) or not set(selection) <= candidate_ids:
            raise DomainError("Review refers to duplicate or absent candidates", 422)
        cues = [
            Cue(id=c.id, prompt=c.prompt, visible_chunks=c.visible_chunks, reviewed=True)
            for c in prepared.candidates
            if c.id in selection
        ]
        target = TargetInput(
            source_id=phrase.source_id,
            kind=phrase.kind,
            meaning_key=phrase.meaning_key,
            recall_expression=phrase.recall_expression,
            text=phrase.text,
            purpose=phrase.purpose,
            chunks=prepared.chunks,
            required_chunks=prepared.required_chunks,
            cues=cues,
            explanations={source.native_language: prepared.explanation},
            concepts=prepared.concepts,
            acceptable_equivalents=prepared.acceptable_equivalents,
            slots=prepared.slots,
            reference_audio_id=phrase.reference_audio_id,
        )
        check_candidates(prepared, target, cues)
        targets.append(target)
    return ManualPackage(
        namespace=source.namespace,
        language=source.language,
        source_revision=source.source_revision,
        targets=targets,
        lessons=source.lessons,
        dialogues=source.dialogues,
    )


def check_candidates(prepared, target, cues):
    for candidate, cue in ((c, q) for c in prepared.candidates for q in cues if c.id == q.id):
        if candidate.target_leak or not has_visible_hint(target, cue):
            raise DomainError("Candidate leaks its target or lacks visible phrase anchors", 422)
        if target.kind != "fixed_phrase" and set(target.required_chunks) & set(
            cue.visible_chunks,
        ):
            raise DomainError("Word/frame cues must hide all required chunks", 422)
        for chunk in target.chunks:
            if chunk.id in cue.visible_chunks and chunk.text.strip() not in cue.prompt:
                raise DomainError("Cue prompt must contain its claimed visible chunks", 422)
    hidden_counts = {len(set(target.required_chunks) - set(c.visible_chunks)) for c in cues}
    if len(hidden_counts) > 1:
        raise DomainError("Candidate masks must have comparable hidden required-chunk counts", 422)
    masks = {tuple(sorted(c.visible_chunks)) for c in cues}
    if len(masks) != len(cues):
        raise DomainError("Cosmetic variants must not duplicate the same mask", 422)
