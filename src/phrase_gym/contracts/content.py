from typing import Literal
from uuid import UUID

from pydantic import Field, model_validator

from phrase_gym.contracts.base import Contract, Identifier, Language, Text


class Chunk(Contract):
    id: Identifier
    text: Text


class Cue(Contract):
    id: Identifier
    prompt: Text
    visible_chunks: list[Identifier]
    reviewed: bool = False
    audio_id: UUID | None = None


class Explanation(Contract):
    meaning: Text
    usage: Text | None = None
    breakdown: list[str] = Field(default_factory=list, max_length=50)


class TargetInput(Contract):
    source_id: Identifier
    kind: Literal["fixed_phrase", "word_sense", "frame"]
    meaning_key: Identifier
    recall_expression: Text
    text: Text
    purpose: Text
    chunks: list[Chunk] = Field(min_length=1, max_length=100)
    required_chunks: list[Identifier] = Field(min_length=1, max_length=100)
    cues: list[Cue] = Field(default_factory=list, max_length=50)
    explanations: dict[Language, Explanation] = Field(default_factory=dict)
    concepts: list[str] = Field(default_factory=list, max_length=100)
    acceptable_equivalents: list[Text] = Field(default_factory=list, max_length=100)
    slots: dict[Identifier, list[Text]] = Field(default_factory=dict)
    reference_audio_id: UUID | None = None

    @model_validator(mode="after")
    def check_chunks(self) -> "TargetInput":
        ids = [chunk.id for chunk in self.chunks]
        if len(ids) != len(set(ids)) or "".join(c.text for c in self.chunks) != self.text:
            raise ValueError("Chunk IDs must be unique and chunks must concatenate to the phrase")
        if not set(self.slots) <= set(ids):
            raise ValueError("Frame slots must reference existing chunks")
        required = set(self.required_chunks)
        if len(required) != len(self.required_chunks) or not required <= set(ids):
            raise ValueError("Required chunks must be distinct existing chunk IDs")
        if len({cue.id for cue in self.cues}) != len(self.cues):
            raise ValueError("Cue IDs must be unique")
        for cue in self.cues:
            visible = set(cue.visible_chunks)
            if len(visible) != len(cue.visible_chunks) or not visible <= set(ids):
                raise ValueError("Visible chunks must be distinct existing chunk IDs")
            if not required - visible:
                raise ValueError("A retrieval cue must hide required material")
            if not cue.reviewed:
                raise ValueError("Manual packages accept reviewed cues only")
        return self


class LessonInput(Contract):
    source_id: Identifier
    title: Text
    target_source_ids: list[Identifier] = Field(min_length=1, max_length=1000)


class DialogueTurn(Contract):
    speaker: Text
    text: Text
    target_source_id: Identifier | None = None
    audio_id: UUID | None = None


class DialogueInput(Contract):
    source_id: Identifier
    title: Text
    turns: list[DialogueTurn] = Field(min_length=2, max_length=100)


class ManualPackage(Contract):
    schema_version: Literal[1] = 1
    namespace: Identifier
    language: Language
    source_revision: Text
    targets: list[TargetInput] = Field(min_length=1, max_length=1000)
    lessons: list[LessonInput] = Field(default_factory=list, max_length=100)
    dialogues: list[DialogueInput] = Field(default_factory=list, max_length=100)

    @model_validator(mode="after")
    def check_links(self) -> "ManualPackage":
        ids = {t.source_id for t in self.targets}
        if len(ids) != len(self.targets):
            raise ValueError("Target source IDs must be unique")
        for collection in (self.lessons, self.dialogues):
            if len({item.source_id for item in collection}) != len(collection):
                raise ValueError("Lesson/dialogue source IDs must be unique within their kind")
        for lesson in self.lessons:
            members = lesson.target_source_ids
            if len(members) != len(set(members)) or not set(members) <= ids:
                raise ValueError(
                    "Lesson members must be distinct target source IDs in this package",
                )
        self.check_dialogue_targets(ids)
        return self

    def check_dialogue_targets(self, ids: set[str]) -> None:
        for dialogue in self.dialogues:
            for turn in dialogue.turns:
                if turn.target_source_id is not None and turn.target_source_id not in ids:
                    raise ValueError("Dialogue target must exist in this package")
                if turn.target_source_id is not None:
                    target = next(t for t in self.targets if t.source_id == turn.target_source_id)
                    if target.text != turn.text:
                        raise ValueError("Dialogue target text must match its complete phrase")
