"""Source intent and withheld preparation drafts, independent of publication."""

from typing import Literal
from uuid import UUID

from pydantic import Field, model_validator

from phrase_gym.contracts.base import Contract, Identifier, Language, Text
from phrase_gym.contracts.content import Chunk, DialogueInput, Explanation, LessonInput


class SourcePhrase(Contract):
    source_id: Identifier
    kind: Literal["fixed_phrase", "word_sense", "frame"] = "fixed_phrase"
    meaning_key: Identifier
    selected_meaning: Text | None = None
    recall_expression: Text
    text: Text
    purpose: Text
    context: Text | None = None
    competing_expressions: list[Text] = Field(default_factory=list, max_length=30)
    reference_audio_id: UUID | None = None


class SourcePackage(Contract):
    namespace: Identifier
    language: Language
    native_language: Language
    source_revision: Text
    phrases: list[SourcePhrase] = Field(min_length=1, max_length=100)
    lessons: list[LessonInput] = Field(default_factory=list, max_length=100)
    dialogues: list[DialogueInput] = Field(default_factory=list, max_length=100)

    @model_validator(mode="after")
    def unique_phrases(self) -> "SourcePackage":
        ids = {phrase.source_id for phrase in self.phrases}
        if len(ids) != len(self.phrases):
            raise ValueError("Source phrase IDs must be unique")
        if any(not set(lesson.target_source_ids) <= ids for lesson in self.lessons):
            raise ValueError("Lesson refers to absent source phrases")
        for dialogue in self.dialogues:
            for turn in dialogue.turns:
                if turn.target_source_id and turn.target_source_id not in ids:
                    raise ValueError("Dialogue refers to absent source phrases")
        return self


class Candidate(Contract):
    id: Identifier
    prompt: Text
    visible_chunks: list[Identifier]
    competing_completions: list[Text] = Field(min_length=1, max_length=10)
    distinguishing_anchors: Text
    recalled_material: Text
    supplied_support: Text
    target_leak: bool


class PreparedPhrase(Contract):
    source_id: Identifier
    chunks: list[Chunk] = Field(min_length=1, max_length=100)
    required_chunks: list[Identifier] = Field(min_length=1, max_length=100)
    candidates: list[Candidate] = Field(default_factory=list, max_length=20)
    uncovered_chunks: dict[Identifier, Text] = Field(default_factory=dict)
    no_reliable_cue_reason: Text | None = None
    explanation: Explanation
    concepts: list[Text] = Field(default_factory=list, max_length=100)
    acceptable_equivalents: list[Text] = Field(default_factory=list, max_length=30)
    slots: dict[Identifier, list[Text]] = Field(default_factory=dict)


class PreparationDraft(Contract):
    phrases: list[PreparedPhrase] = Field(min_length=1, max_length=100)


class CandidateReview(Contract):
    source_id: Identifier
    candidate_id: Identifier
    acceptable: bool
    reason: Text


class ModelReview(Contract):
    candidates: list[CandidateReview]
    help_errors: dict[Identifier, Text]
    meaning_errors: dict[Identifier, Text]


class HumanReview(Contract):
    input_hash: str = Field(pattern="^[a-f0-9]{64}$")
    draft_hash: str = Field(pattern="^[a-f0-9]{64}$")
    reviewer: Text
    accepted_cues: dict[Identifier, list[Identifier]]
    help_reviewed: Literal[True]
    meaning_search_equivalents_reviewed: Literal[True]
    audio_reviewed: Literal[True]
    notes: Text


class PreparationJob(Contract):
    id: UUID
    status: Literal["queued", "leased", "failed", "awaiting_review", "ready"]
    input_hash: str
    attempts: int
    error: str | None
    result: dict | None
    published_import_id: UUID | None
