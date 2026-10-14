"""Explicit selected-note mappings live outside the read-only source vault."""

from pathlib import Path
from typing import Literal

from pydantic import Field

from phrase_gym.contracts.base import Contract, Identifier, Language, Text
from phrase_gym.contracts.content import DialogueInput, LessonInput
from phrase_gym.contracts.media import MediaMetadata


class VaultSelection(Contract):
    source_id: Identifier
    note: Path
    block_id: Identifier | None = None
    source_audio_name: Text | None = None
    meaning_key: Identifier
    recall_expression: Text
    purpose: Text
    kind: Literal["fixed_phrase", "word_sense", "frame"] = "fixed_phrase"
    context: str | None = None
    attachment: Path | None = None
    audio_metadata: MediaMetadata | None = None
    mime_type: str | None = None


class VaultAsset(Contract):
    path: Path
    metadata: MediaMetadata
    mime_type: str


class VaultManifest(Contract):
    namespace: Identifier
    language: Language
    native_language: Language
    include_dialogues: bool = False
    assets: dict[str, VaultAsset] = Field(default_factory=dict)
    selections: list[VaultSelection] = Field(min_length=1, max_length=100)
    lessons: list[LessonInput] = Field(default_factory=list)
    dialogues: list[DialogueInput] = Field(default_factory=list)
