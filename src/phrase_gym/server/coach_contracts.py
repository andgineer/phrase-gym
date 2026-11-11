from typing import Literal

from pydantic import Field

from phrase_gym.server.import_contracts import IDENTITY_PATTERN, ImportPhrase, StrictModel

MAX_TYPED_LENGTH = 1000
COACH_SOURCE_ID = "phrase-gym-coach"
COACH_SOURCE_LABEL = "Added with Coach"
COACH_LESSON_ID = "added-with-coach"


class CoachRequest(StrictModel):
    text: str = Field(min_length=1, max_length=MAX_TYPED_LENGTH)


class CoachSubmission(StrictModel):
    request_id: str
    outcome: Literal["saved", "waiting", "in_library"]


class CoachWork(StrictModel):
    package_id: str
    text: str


class CoachResult(StrictModel):
    result_id: str = Field(pattern=IDENTITY_PATTERN)
    phrase: ImportPhrase
    reference: str


class CoachFailure(StrictModel):
    error: str = Field(min_length=1, max_length=1000)
