from datetime import datetime
from typing import Literal

from pydantic import Field, field_validator, model_validator

from phrase_gym.server.import_contracts import Cue, StrictModel

PAGE_LIMIT = 100
MAX_DAILY_LIMIT = 500
PracticeMode = Literal["daily", "extra", "practice"]
CardKind = Literal["first_study", "recall"]
# Take identities become file names: no separators or dots, and lowercase so that IDs differing
# only by case cannot share a file on case-insensitive filesystems.
STORED_ID_PATTERN = r"^[a-z0-9][a-z0-9_-]{7,99}$"


class RecordingUpload(StrictModel):
    id: str = Field(pattern=STORED_ID_PATTERN)
    phrase_id: str = Field(pattern=STORED_ID_PATTERN)
    view_token: str = Field(pattern=STORED_ID_PATTERN)
    mode: PracticeMode
    kind: CardKind
    cue: Cue | None = None
    hint: str | None = Field(default=None, max_length=100)
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


class Settings(StrictModel):
    native_language: Literal["ru", "en"]
    learning_timezone: str = Field(min_length=1, max_length=64)
    card_language: Literal["en"]
    help_language: Literal["en"]
    daily_due_limit: int = Field(ge=0, le=MAX_DAILY_LIMIT)
    daily_new_limit: int = Field(ge=0, le=MAX_DAILY_LIMIT)

    @model_validator(mode="after")
    def some_practice_is_allowed(self) -> "Settings":
        if not self.daily_due_limit and not self.daily_new_limit:
            raise ValueError("daily due and new limits cannot both be zero")
        return self


class RecordingReceipt(StrictModel):
    id: str
    byte_length: int
    sha256: str
    complete: bool


class SavedRecording(StrictModel):
    id: str
    captured_at: str
    learning_date: str
    mime_type: str
    duration_ms: int | None
    complete: bool
    byte_length: int
    url: str
    mode: PracticeMode
    kind: CardKind
    cue: Cue | None
    hint: str | None
    answer: str


class PracticeCounts(StrictModel):
    phrase_count: int
    due_count: int
    new_count: int
    next_due: str | None


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
