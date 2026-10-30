from collections.abc import Iterator
from contextlib import contextmanager

from fastapi import APIRouter, HTTPException

from phrase_gym.server.completion import PracticeCompletion
from phrase_gym.server.daily import DailyPractice
from phrase_gym.server.extra import ExtraPractice
from phrase_gym.server.phrase_practice import PhrasePractice, UnknownPhraseError
from phrase_gym.server.practice_contracts import (
    CompletionRequest,
    CompletionResult,
    ExtraResult,
    OpenDaily,
    OpenExtra,
    OpenPhrasePractice,
    PracticeCard,
    PracticeState,
    ViewAction,
    ViewOperation,
)
from phrase_gym.server.practice_views import PracticeConflictError, UnknownLessonError
from phrase_gym.server.view_actions import ViewActions


@contextmanager
def conflicts() -> Iterator[None]:
    try:
        yield
    except PracticeConflictError as error:
        raise HTTPException(409, str(error)) from error
    except UnknownLessonError as error:
        raise HTTPException(404, "Lesson not found") from error
    except UnknownPhraseError as error:
        raise HTTPException(404, "Phrase not found") from error


def practice_routes(
    daily: DailyPractice,
    extra: ExtraPractice,
    phrase: PhrasePractice,
    actions: ViewActions,
    completion: PracticeCompletion,
) -> APIRouter:
    router = APIRouter(prefix="/api/practice")

    @router.post("/daily")
    def open_daily(request: OpenDaily) -> PracticeState:
        with conflicts():
            return daily.open(request.lesson_id)

    @router.post("/extra")
    def open_extra(request: OpenExtra) -> ExtraResult:
        with conflicts():
            return extra.open(request)

    @router.post("/phrase")
    def open_phrase(request: OpenPhrasePractice) -> PracticeState:
        with conflicts():
            return phrase.open(request.phrase_id)

    @router.post("/reveal")
    def reveal(request: ViewAction) -> PracticeCard:
        with conflicts():
            return actions.reveal(request)

    @router.post("/cue")
    def new_cue(request: ViewOperation) -> PracticeCard:
        with conflicts():
            return actions.new_cue(request)

    @router.post("/hint")
    def hint(request: ViewOperation) -> PracticeCard:
        with conflicts():
            return actions.hint(request)

    @router.post("/complete")
    def complete(request: CompletionRequest) -> CompletionResult:
        with conflicts():
            return completion.complete(request)

    return router
