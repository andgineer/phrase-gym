from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, HttpUrl, field_validator


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)


class Cue(StrictModel):
    situation: str = Field(min_length=5, max_length=500)
    partner_line: str | None = Field(default=None, min_length=1, max_length=300)


class ImportCard(StrictModel):
    item_id: str = Field(pattern=r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$")
    answer: str = Field(min_length=2, max_length=500)
    explanation: str = Field(min_length=5, max_length=1000)
    cues: list[Cue] = Field(min_length=1, max_length=2)
    accepted_variants: list[str] = Field(default_factory=list, max_length=8)
    detailed_explanation: str | None = Field(default=None, max_length=2000)

    @field_validator("accepted_variants")
    @classmethod
    def variants_are_nonempty(cls, values: list[str]) -> list[str]:
        if any(not value.strip() for value in values):
            raise ValueError("accepted variants cannot be empty")
        return values


class PhrasePackage(StrictModel):
    format: Literal["phrase-gym.package"]
    version: Literal[1]
    package_id: str = Field(pattern=r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$")
    source_id: str = Field(pattern=r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$")
    source_label: str = Field(min_length=1, max_length=200)
    source_link: HttpUrl | None = None
    learning_language: Literal["en"]
    card_language: Literal["en"]
    help_language: Literal["en"]
    cards: list[ImportCard] = Field(min_length=1, max_length=20)

    @field_validator("cards")
    @classmethod
    def item_ids_are_unique(cls, cards: list[ImportCard]) -> list[ImportCard]:
        ids = [card.item_id for card in cards]
        if len(ids) != len(set(ids)):
            raise ValueError("card item_id values must be unique within a package")
        return cards


class ImportRequest(StrictModel):
    package: PhrasePackage


class RecordingUpload(StrictModel):
    id: str = Field(min_length=8, max_length=100)
    card_id: str = Field(min_length=8, max_length=100)
    attempt_id: str = Field(min_length=8, max_length=100)
    captured_at: str = Field(min_length=10, max_length=64)
    mime_type: str = Field(min_length=3, max_length=100)
    duration_ms: int | None = Field(default=None, ge=0, le=3_600_000)
    complete: bool
    capture_settings: dict[str, str | int | float | bool] = Field(default_factory=dict)
    audio_base64: str


class CompleteStudy(StrictModel):
    operation_id: str = Field(min_length=8, max_length=100)


class ImportStatus(StrictModel):
    package_id: str
    status: Literal["preparing", "added", "error"]
    error: str | None = None
    added_count: int | None = None


class RecordingReceipt(StrictModel):
    id: str
    byte_length: int
    sha256: str
    complete: bool


class SavedRecording(StrictModel):
    id: str
    attempt_id: str
    captured_at: str
    mime_type: str
    duration_ms: int | None
    complete: bool
    byte_length: int
    url: str


class StudyCard(StrictModel):
    id: str
    source_id: str
    item_id: str
    source_label: str
    source_link: str | None
    answer: str
    explanation: str
    cues: list[Cue]
    accepted_variants: list[str]
    detailed_explanation: str | None
    reference_url: str
    attempt_id: str
    recordings: list[SavedRecording]


class StudyResponse(StrictModel):
    done: bool
    total: int
    studied: int
    message: str | None = None
    card: StudyCard | None = None


class StudyHistoryItem(StrictModel):
    id: str
    answer: str
    source_label: str
    reference_url: str
    recordings: list[SavedRecording]


class StudyCompletion(StrictModel):
    attempt_id: str
    completed_at: str
    graded: Literal[False]
