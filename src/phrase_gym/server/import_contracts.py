from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, HttpUrl, field_validator, model_validator

from phrase_gym.server.search_terms import search_key

IDENTITY_PATTERN = r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$"
MAX_PACKAGE_PHRASES = 50
MAX_HINT_LENGTH = 40
MAX_CONTEXT_LENGTH = 60


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)


class Cue(StrictModel):
    situation: str = Field(min_length=5, max_length=500)
    partner_line: str | None = Field(default=None, min_length=1, max_length=300)


def _distinct(values: list[str], label: str) -> list[str]:
    if any(not value.strip() for value in values):
        raise ValueError(f"{label} cannot be empty")
    folded = [value.casefold() for value in values]
    if len(folded) != len(set(folded)):
        raise ValueError(f"{label} must be distinct")
    return values


class ImportPhrase(StrictModel):
    item_id: str = Field(pattern=IDENTITY_PATTERN)
    text: str = Field(min_length=2, max_length=500)
    explanation: str = Field(min_length=5, max_length=1000)
    cues: list[Cue] = Field(min_length=1, max_length=2)
    recall_hints: list[str] = Field(min_length=2, max_length=6)
    usage_contexts: list[str] = Field(min_length=1, max_length=6)
    accepted_variants: list[str] = Field(default_factory=list, max_length=8)
    detailed_explanation: str | None = Field(default=None, max_length=2000)

    @field_validator("cues")
    @classmethod
    def cues_are_distinct(cls, cues: list[Cue]) -> list[Cue]:
        _distinct([cue.situation for cue in cues], "cue situations")
        return cues

    @field_validator("recall_hints")
    @classmethod
    def hints_are_short_and_distinct(cls, values: list[str]) -> list[str]:
        if any(len(value) > MAX_HINT_LENGTH for value in values):
            raise ValueError(f"recall hints must be at most {MAX_HINT_LENGTH} characters")
        return _distinct(values, "recall hints")

    @field_validator("usage_contexts")
    @classmethod
    def contexts_are_short_and_distinct(cls, values: list[str]) -> list[str]:
        if any(len(value) > MAX_CONTEXT_LENGTH for value in values):
            raise ValueError(f"usage contexts must be at most {MAX_CONTEXT_LENGTH} characters")
        return _distinct(values, "usage contexts")

    @field_validator("accepted_variants")
    @classmethod
    def variants_are_nonempty(cls, values: list[str]) -> list[str]:
        if any(not value.strip() for value in values):
            raise ValueError("accepted variants cannot be empty")
        return values

    @model_validator(mode="after")
    def hints_do_not_repeat_the_answer(self) -> "ImportPhrase":
        answer = search_key(self.text)
        if any(search_key(hint) == answer for hint in self.recall_hints):
            raise ValueError("recall hints cannot repeat the complete phrase")
        return self


class ImportLesson(StrictModel):
    lesson_id: str = Field(pattern=IDENTITY_PATTERN)
    title: str = Field(min_length=1, max_length=200)
    phrases: list[ImportPhrase] = Field(min_length=1, max_length=MAX_PACKAGE_PHRASES)


class PhrasePackage(StrictModel):
    format: Literal["phrase-gym.package"]
    version: Literal[2]
    package_id: str = Field(pattern=IDENTITY_PATTERN)
    source_id: str = Field(pattern=IDENTITY_PATTERN)
    source_label: str = Field(min_length=1, max_length=200)
    source_link: HttpUrl | None = None
    learning_language: Literal["en"]
    card_language: Literal["en"]
    help_language: Literal["en"]
    lessons: list[ImportLesson] = Field(min_length=1, max_length=20)

    @field_validator("lessons")
    @classmethod
    def identities_are_unique(cls, lessons: list[ImportLesson]) -> list[ImportLesson]:
        lesson_ids = [lesson.lesson_id for lesson in lessons]
        if len(lesson_ids) != len(set(lesson_ids)):
            raise ValueError("lesson_id values must be unique within a package")
        item_ids = [phrase.item_id for lesson in lessons for phrase in lesson.phrases]
        if len(item_ids) != len(set(item_ids)):
            raise ValueError("phrase item_id values must be unique within a package")
        if len(item_ids) > MAX_PACKAGE_PHRASES:
            raise ValueError(f"a package may contain at most {MAX_PACKAGE_PHRASES} phrases")
        return lessons

    def phrases(self) -> list[tuple[ImportLesson, ImportPhrase]]:
        return [(lesson, phrase) for lesson in self.lessons for phrase in lesson.phrases]


class ImportRequest(StrictModel):
    package: PhrasePackage
    references: dict[str, str] | None = None


class ImportStatus(StrictModel):
    package_id: str
    status: Literal["pending", "preparing", "added", "error"]
    error: str | None = None
    added_count: int | None = None


class CoachWork(StrictModel):
    package_id: str
    package: PhrasePackage


class CoachResult(StrictModel):
    result_id: str = Field(pattern=IDENTITY_PATTERN)
    references: dict[str, str]


class CoachFailure(StrictModel):
    error: str = Field(min_length=1, max_length=1000)
