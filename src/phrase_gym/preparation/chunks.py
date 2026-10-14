"""Reconcile stored chunk identities without relabelling retained recall evidence."""

import hashlib
import json
from collections import defaultdict

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.content import TargetInput
from phrase_gym.contracts.preparation import PreparationDraft, PreparedPhrase, SourcePackage
from phrase_gym.practice.tables import ChunkHistoryRow
from phrase_gym.tables import SourceItem, TargetRevision

Material = tuple[str, bool, tuple[str, ...], int]


def material_keys(content: TargetInput | PreparedPhrase) -> dict[str, Material]:
    occurrences = defaultdict(int)
    result = {}
    for chunk in content.chunks:
        material = (
            chunk.text,
            chunk.id in content.required_chunks,
            tuple(sorted(content.slots.get(chunk.id, []))),
        )
        occurrence = occurrences[material]
        occurrences[material] += 1
        result[chunk.id] = (*material, occurrence)
    return result


def retained_materials(transaction: Session, target_id: str):
    revisions = transaction.scalars(
        select(TargetRevision)
        .where(
            TargetRevision.target_id == target_id,
        )
        .order_by(TargetRevision.revision.desc()),
    ).all()
    identities: dict[str, set[Material]] = defaultdict(set)
    current = {}
    for index, row in enumerate(revisions):
        keys = material_keys(TargetInput.model_validate_json(row.payload))
        if index == 0:
            current = keys
        for chunk_id, key in keys.items():
            identities[chunk_id].add(key)
    reserved = set(identities)
    reserved.update(
        transaction.scalars(
            select(ChunkHistoryRow.chunk_id).where(
                ChunkHistoryRow.target_id == target_id,
            ),
        ),
    )
    return identities, current, reserved


def choose_identity(key: Material, proposed: str, identities, current, reserved) -> str:
    existing = [chunk_id for chunk_id, materials in identities.items() if materials == {key}]
    if existing:
        return min(existing, key=lambda chunk_id: (current.get(chunk_id) != key, chunk_id))
    if proposed not in reserved:
        return proposed
    sequence = 0
    while True:
        encoded = json.dumps([key, sequence], ensure_ascii=False).encode()
        chunk_id = "material." + hashlib.sha256(encoded).hexdigest()[:32]
        if chunk_id not in reserved:
            return chunk_id
        sequence += 1


def reconcile_draft(transaction: Session, source: SourcePackage, draft: PreparationDraft):
    normalized = draft.model_copy(deep=True)
    mappings = {}
    for phrase in normalized.phrases:
        item = transaction.get(SourceItem, (source.namespace, phrase.source_id))
        identities, current, reserved = (
            retained_materials(transaction, item.target_id) if item else ({}, {}, set())
        )
        mapping = {}
        for chunk_id, key in material_keys(phrase).items():
            selected = choose_identity(key, chunk_id, identities, current, reserved)
            mapping[chunk_id] = selected
            reserved.add(selected)
        remap_phrase(phrase, mapping)
        mappings[phrase.source_id] = mapping
    return normalized, mappings


def remap_phrase(phrase: PreparedPhrase, mapping: dict[str, str]) -> None:
    for chunk in phrase.chunks:
        chunk.id = mapping[chunk.id]
    phrase.required_chunks = [mapping[chunk_id] for chunk_id in phrase.required_chunks]
    for candidate in phrase.candidates:
        candidate.visible_chunks = [mapping[chunk_id] for chunk_id in candidate.visible_chunks]
    phrase.uncovered_chunks = {
        mapping[chunk_id]: reason for chunk_id, reason in phrase.uncovered_chunks.items()
    }
    phrase.slots = {mapping[chunk_id]: values for chunk_id, values in phrase.slots.items()}
