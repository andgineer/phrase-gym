from datetime import datetime
from typing import Literal

from pydantic import Field, field_validator

from phrase_gym.server.import_contracts import Cue, StrictModel

PAGE_LIMIT = 100
# Take identities become file names, so they exclude path separators and dots.
STORED_ID_PATTERN = r"^[A-Za-z0-9][A-Za-z0-9_-]{7,99}$"


class RecordingUpload(StrictModel):
    id: str = Field(pattern=STORED_ID_PATTERN)
    phrase_id: str = Field(pattern=STORED_ID_PATTERN)
    attempt_id: str = Field(pattern=STORED_ID_PATTERN)
    captured_at: str = Field(min_length=10, max_length=64)
    mime_type: str = Field(min_length=3, max_length=100)
    duration_ms: int | None = Field(default=None, ge=0, le=3_600_000)
    complete: bool
    capture_settings: dict[str, str | int | float | bool] = Field(default_factory=dict)
    audio_base64: str

    @field_validator("captured_at")
    @classmethod
    def captured_at_has_timezone(cls, value: str) -> str:
        if datetime.fromisoformat(value).tzinfo is None:
            raise ValueError("captured_at must include a timezone")
        return value


class CompleteStudy(StrictModel):
    operation_id: str = Field(min_length=8, max_length=100)


class ReviewRequest(CompleteStudy):
    rating: Literal["not_yet", "with_effort", "recalled"]
    response_mode: Literal["spoken", "silent"]


class AttemptAction(StrictModel):
    response_mode: Literal["spoken", "silent"] | None = None


class Settings(StrictModel):
    native_language: Literal["ru", "en"]
    learning_timezone: str = Field(min_length=1, max_length=64)
    card_language: Literal["en"]
    help_language: Literal["en"]


class RecordingReceipt(StrictModel):
    id: str
    byte_length: int
    sha256: str
    complete: bool


class SavedRecording(StrictModel):
    id: str
    attempt_id: str
    captured_at: str
    learning_date: str
    mime_type: str
    duration_ms: int | None
    complete: bool
    byte_length: int
    url: str
    mode: Literal["study", "review", "free"]
    cue: Cue | None
    hint: str | None
    answer: str


class StudyCard(StrictModel):
    id: str
    source_id: str
    item_id: str
    source_label: str
    source_link: str | None
    text: str
    explanation: str
    cues: list[Cue]
    accepted_variants: list[str]
    detailed_explanation: str | None
    reference_url: str
    attempt_id: str
    recordings: list[SavedRecording]
    kind: Literal["study", "review", "free"] = "study"
    revealed: bool = True
    cue: Cue | None = None


class StudyResponse(StrictModel):
    done: bool
    total: int
    studied: int
    message: str | None = None
    card: StudyCard | None = None
    due_count: int = 0
    new_count: int = 0
    next_return: str | None = None


class StudyCompletion(StrictModel):
    attempt_id: str
    completed_at: str
    graded: bool


class PracticeCounts(StrictModel):
    phrase_count: int
    due_count: int
    new_count: int


class LessonChoice(PracticeCounts):
    id: str
    title: str
    source_label: str


class LessonPage(StrictModel):
    items: list[LessonChoice]
    next_offset: int | None
    all_lessons: PracticeCounts


class PhraseSummary(StrictModel):
    id: str
    text: str
    usage_contexts: list[str]
    lesson_title: str


class PhrasePage(StrictModel):
    items: list[PhraseSummary]
    next_offset: int | None


class PhraseDetails(StrictModel):
    id: str
    text: str
    explanation: str
    detailed_explanation: str | None
    usage_contexts: list[str]
    accepted_variants: list[str]
    lesson_id: str
    lesson_title: str
    source_label: str
    source_link: str | None
    reference_url: str
    studied: bool
    due_date: str | None


class RecordingPage(StrictModel):
    items: list[SavedRecording]
    next_offset: int | None


class CleanupStatus(StrictModel):
    pending: int
    error: str | None


class DeletionResult(StrictModel):
    deleted_text: str
    cleanup: CleanupStatus
