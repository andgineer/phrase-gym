from collections.abc import Iterator
from contextlib import contextmanager

from fastapi import APIRouter, HTTPException

from phrase_gym.server.completion import PracticeCompletion
from phrase_gym.server.daily import DailyPractice, UnknownLessonError
from phrase_gym.server.practice_contracts import (
    CompletionRequest,
    CompletionResult,
    OpenDaily,
    PracticeCard,
    PracticeState,
    ViewAction,
    ViewOperation,
)
from phrase_gym.server.practice_views import PracticeConflictError
from phrase_gym.server.view_actions import ViewActions


@contextmanager
def conflicts() -> Iterator[None]:
    try:
        yield
    except PracticeConflictError as error:
        raise HTTPException(409, str(error)) from error


def practice_routes(
    daily: DailyPractice,
    actions: ViewActions,
    completion: PracticeCompletion,
) -> APIRouter:
    router = APIRouter(prefix="/api/practice")

    @router.post("/daily")
    def open_daily(request: OpenDaily) -> PracticeState:
        try:
            return daily.open(request.lesson_id)
        except UnknownLessonError as error:
            raise HTTPException(404, "Lesson not found") from error

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
