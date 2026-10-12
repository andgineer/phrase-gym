from datetime import datetime
from typing import Literal
from uuid import UUID

from pydantic import Field

from phrase_gym.contracts.base import Contract, Identifier, Language
from phrase_gym.contracts.library import Phrase

SessionStatus = Literal["active", "paused", "superseded"]


class SessionStart(Contract):
    id: UUID
    language: Language
    lesson_id: UUID | None = None
    takeover_session_id: UUID | None = None


class PlannedTarget(Contract):
    phrase: Phrase
    cue_id: Identifier
    hidden_chunks: list[Identifier]
    review_group_id: UUID
    schedule_version: int = Field(ge=0)
    eligibility: Literal["new", "due"]


class SessionManifest(Contract):
    schema_version: Literal[1] = 1
    planner_version: Literal["due-new-v1"] = "due-new-v1"
    language: Language
    lesson_id: UUID | None = None
    targets: list[PlannedTarget]
    required_media_ids: list[UUID]


class PracticeSession(Contract):
    id: UUID
    status: SessionStatus
    version: int = Field(ge=1)
    created_at: datetime
    updated_at: datetime
    manifest: SessionManifest


class SessionTransition(Contract):
    expected_version: int = Field(ge=1)
    status: Literal["active", "paused"]
