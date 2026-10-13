import json
from datetime import datetime
from typing import Literal
from uuid import UUID

from pydantic import AwareDatetime, Field, model_validator
from pydantic.json_schema import SkipJsonSchema

from phrase_gym.contracts.base import Contract, Identifier

RecallRating = Literal["not_yet", "with_effort", "recalled"]
Modality = Literal["self_report", "recorded", "unrecorded"]


class PracticeInput(Contract):
    id: UUID
    sequence: int = Field(ge=1)
    step: int = Field(ge=0)
    occurred_at: AwareDatetime
    kind: Literal[
        "present",
        "respond",
        "capture_complete",
        "answer",
        "explain",
        "reference",
        "rate",
        "failure",
        "report",
        "detour",
        "dialogue_exposure",
        "dialogue_begin",
    ]
    modality: Modality = "self_report"
    rating: RecallRating | None = None
    reason: str | None = Field(default=None, max_length=1000)
    legacy_deferral: SkipJsonSchema[bool | None] = Field(
        default=None,
        validation_alias="defer_repairs",
        exclude=True,
        strict=True,
    )

    @model_validator(mode="after")
    def validate_kind(self) -> "PracticeInput":
        if (self.kind == "rate") != (self.rating is not None):
            raise ValueError("Only a rate event supplies a rating")
        if self.kind == "report" and not self.reason:
            raise ValueError("Cue reports require a reason")
        if self.legacy_deferral and self.kind != "rate":
            raise ValueError("The retired deferral flag is valid only on historical ratings")
        return self

    def retained_payload(self) -> str:
        body = self.model_dump(mode="json")
        if self.legacy_deferral is not None:
            body["defer_repairs"] = self.legacy_deferral
        return json.dumps(body)


class OriginalResponse(Contract):
    modality: Modality
    supported: bool
    occurred_at: datetime


class AttemptEvidence(Contract):
    id: UUID
    step: int
    target_id: UUID
    revision: int
    cue_id: Identifier
    review_group_id: UUID
    repair: bool
    supported: bool = False
    original: OriginalResponse | None = None
    revealed: bool = False
    expanded: bool = False
    technical_failure: bool = False
    capture_pending: bool = False
    rating: RecallRating | None = None
    presented_at: datetime
    finalized_at: datetime | None = None
    eligibility_reason: str | None = None


class EventReceipt(Contract):
    id: UUID
    sequence: int
    attempt_id: UUID
    received_at: datetime
    occurred_at: datetime
    clock_adjusted: bool
    capture_supported: bool | None = None
    schedule_changed: bool = False
    schedule_version: int | None = None
    due_at: datetime | None = None
    eligibility_reason: str | None = None


class EventBatch(Contract):
    events: list[PracticeInput] = Field(min_length=1, max_length=100)


class RecordingInput(Contract):
    id: UUID
    target_id: UUID
    revision: int = Field(ge=1)
    attempt_id: UUID | None = None
    exposure_id: UUID | None = None
    capture_event_id: UUID | None = None
    captured_at: AwareDatetime
    mime_type: str = Field(min_length=1, max_length=100)
    duration_ms: int | None = Field(default=None, ge=0)
    complete: bool
    capture_settings: dict[str, str | int | float | bool] = Field(default_factory=dict)
    audio_base64: str = Field(min_length=1, max_length=17000000)

    @model_validator(mode="after")
    def validate_context(self) -> "RecordingInput":
        if (self.attempt_id is None) == (self.exposure_id is None):
            raise ValueError("A take links to exactly one attempt or lookup exposure")
        return self


class RecordingReceipt(Contract):
    id: UUID
    sha256: str
    byte_length: int
    captured_at: datetime
    received_at: datetime
    complete: bool
