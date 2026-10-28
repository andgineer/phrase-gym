from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import AttemptAction, ReviewRequest, StudyCard, StudyCompletion
from phrase_gym.server.study import StudyService


def review_routes(service: StudyService) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.post("/attempts/{attempt_id}/reveal")
    def reveal(attempt_id: str, request: AttemptAction) -> StudyCard:
        card = service.reveal(attempt_id, request.response_mode or "silent")
        if not card:
            raise HTTPException(404, "Active recall attempt not found")
        return card

    @router.post("/attempts/{attempt_id}/review")
    def review(attempt_id: str, request: ReviewRequest) -> StudyCompletion:
        try:
            result = service.complete(
                attempt_id,
                request.operation_id,
                request.rating,
                request.response_mode,
            )
        except ValueError as error:
            raise HTTPException(409, str(error)) from error
        if not result:
            raise HTTPException(404, "Recall attempt not found")
        return result

    return router
