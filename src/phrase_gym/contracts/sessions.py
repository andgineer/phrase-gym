from datetime import datetime
from typing import Literal
from uuid import UUID

from pydantic import Field

from phrase_gym.contracts.base import Contract, Identifier, Language
from phrase_gym.contracts.library import Dialogue, Phrase
from phrase_gym.contracts.practice import AttemptEvidence, EventReceipt

SessionStatus = Literal["active", "paused", "superseded", "completed"]


class SessionStart(Contract):
    id: UUID
    language: Language
    lesson_id: UUID | None = None
    dialogue_id: UUID | None = None
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
    planner_version: Literal["due-new-v1", "due-new-v2"] = "due-new-v2"
    policy_version: str = "fsrs-6.3.2-retention90-v1"
    language: Language
    lesson_id: UUID | None = None
    targets: list[PlannedTarget]
    required_media_ids: list[UUID]
    dialogue: Dialogue | None = None


class SessionStep(Contract):
    target_index: int
    repair: bool = False
    dialogue_turn: int | None = None
    introduced: bool = False


class PracticeSession(Contract):
    id: UUID
    status: SessionStatus
    version: int = Field(ge=1)
    created_at: datetime
    updated_at: datetime
    manifest: SessionManifest
    cursor: int
    last_sequence: int
    queue: list[SessionStep]
    attempts: list[AttemptEvidence]
    dialogue_exposures: list[datetime] = Field(default_factory=list)


class SessionTransition(Contract):
    expected_version: int = Field(ge=1)
    status: Literal["active", "paused"]


class PracticeBatchReceipt(Contract):
    receipts: list[EventReceipt]
    session: PracticeSession
