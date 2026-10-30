from typing import Annotated, Literal

from pydantic import Field

from phrase_gym.server.contracts import CardKind, PracticeMode, SavedRecording
from phrase_gym.server.import_contracts import Cue, StrictModel

Rating = Literal["not_yet", "effort", "got_it"]
Outcome = Literal["studied", "not_yet", "effort", "got_it", "skipped"]
OperationId = Annotated[str, Field(min_length=8, max_length=100, pattern=r"^[A-Za-z0-9_-]+$")]


class CardAnswer(StrictModel):
    text: str
    explanation: str
    detailed_explanation: str | None
    accepted_variants: list[str]
    usage_contexts: list[str]
    reference_url: str


class PracticeCard(StrictModel):
    view_token: str
    phrase_id: str
    mode: PracticeMode
    kind: CardKind
    revealed: bool
    cue: Cue | None
    other_cue_available: bool
    hint: str | None
    answer: CardAnswer | None
    recordings: list[SavedRecording]


class PracticeProgress(StrictModel):
    due_completed: int
    due_allowance: int
    new_completed: int
    new_allowance: int


class PracticeState(StrictModel):
    card: PracticeCard | None
    progress: PracticeProgress
    message: str | None


class OpenDaily(StrictModel):
    lesson_id: str | None = Field(default=None, max_length=100)


class ViewAction(StrictModel):
    view_token: str = Field(min_length=8, max_length=100)
    phrase_id: str = Field(min_length=8, max_length=100)


class ViewOperation(ViewAction):
    operation_id: OperationId


class CompletionRequest(ViewAction):
    completion_id: OperationId
    outcome: Outcome


class CompletionReceipt(StrictModel):
    completion_id: str
    phrase_id: str
    learning_date: str
    mode: PracticeMode
    kind: CardKind
    rating: Rating | None
    skipped: bool
    fsrs_updated: bool


class CompletionResult(StrictModel):
    receipt: CompletionReceipt
    replayed: bool
    state: PracticeState | None
