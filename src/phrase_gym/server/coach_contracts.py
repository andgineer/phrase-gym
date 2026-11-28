from typing import Literal

from pydantic import Field, model_validator

from phrase_gym.server.import_contracts import IDENTITY_PATTERN, ImportPhrase, StrictModel
from phrase_gym.server.languages import LearningLanguage, NativeLanguage
from phrase_gym.server.search_terms import search_key
from phrase_gym.server.sentence_gaps import gap_around, realizes, shows_target

MAX_TYPED_LENGTH = 1000
COACH_SOURCE_ID = "phrase-gym-coach"
COACH_SOURCE_LABEL = "Added with Coach"
COACH_LESSON_ID = "added-with-coach"


class CoachRequest(StrictModel):
    text: str = Field(min_length=1, max_length=MAX_TYPED_LENGTH)
    learning_language: LearningLanguage


class CoachSubmission(StrictModel):
    request_id: str
    outcome: Literal["saved", "waiting", "in_library", "rejected"]


class CoachWork(StrictModel):
    package_id: str
    text: str
    learning_language: LearningLanguage
    native_language: NativeLanguage
    translation: bool


class CoachCorrection(StrictModel):
    original: str = Field(min_length=1, max_length=200)
    corrected: str = Field(min_length=1, max_length=200)


class BasicCard(StrictModel):
    """Wording practice for a confirmed target, without authored meaning or examples."""

    target: str = Field(min_length=1, max_length=200)
    text: str = Field(min_length=1, max_length=500)
    cue: str = Field(min_length=1, max_length=500)
    instruction: str = Field(min_length=1, max_length=200)
    correction: CoachCorrection | None = None

    @model_validator(mode="after")
    def cue_conceals_the_answer(self) -> "BasicCard":
        target = search_key(self.target)
        if target in search_key(self.cue) or target in search_key(self.instruction):
            raise ValueError("the cue and instruction must conceal the target")
        if gap := gap_around(self.cue):
            before, after = gap
            filled = self.text[len(before) : len(self.text) - len(after)]
            fits = self.text.startswith(before) and self.text.endswith(after)
            if not fits or not realizes(filled, self.target) or shows_target(self.cue, self.target):
                raise ValueError("the revealed text must fill the gap with a form of the target")
        elif target not in search_key(self.text):
            raise ValueError("the revealed text must contain the target")
        if self.correction and self.correction.corrected != self.target:
            raise ValueError("a correction must end in the confirmed target")
        return self


class CoachResult(StrictModel):
    result_id: str = Field(pattern=IDENTITY_PATTERN)
    phrase: ImportPhrase | None = None
    basic: BasicCard | None = None
    # A prepared phrase card's own correction of the input into its studied target; a basic card
    # carries its correction itself.
    correction: CoachCorrection | None = None
    target: str | None = Field(default=None, min_length=1, max_length=200)
    reference: str

    @model_validator(mode="after")
    def one_card(self) -> "CoachResult":
        if (self.phrase is None) == (self.basic is None):
            raise ValueError("a result carries either phrase or basic")
        if self.correction and self.phrase is None:
            raise ValueError("only a phrase result carries a separate correction")
        if self.correction and self.correction.original == self.correction.corrected:
            raise ValueError("a correction must change the wording")
        if self.correction and (
            self.target is None
            or search_key(self.correction.corrected) not in search_key(self.target)
        ):
            raise ValueError("the corrected wording must be the target or whole words of it")
        return self

    def corrected(self) -> CoachCorrection | None:
        return self.basic.correction if self.basic else self.correction


class CoachFailure(StrictModel):
    error: str = Field(min_length=1, max_length=1000)
    # False records a language decision about the input rather than a service failure.
    retry_helps: bool = True


class CoachOutcome(StrictModel):
    request_id: str
    text: str
    learning_language: LearningLanguage
    outcome: Literal["corrected", "rejected", "failed"]
    reason: str | None
    correction: CoachCorrection | None


class CoachInbox(StrictModel):
    items: list[CoachOutcome]
