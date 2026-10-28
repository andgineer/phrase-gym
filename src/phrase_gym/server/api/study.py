from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse

from phrase_gym.server.contracts import (
    CompleteStudy,
    StudyCard,
    StudyCompletion,
    StudyResponse,
)
from phrase_gym.server.study import StudyService


def study_routes(service: StudyService) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.post("/study/current")
    def current_study() -> StudyResponse:
        return service.current()

    @router.post("/attempts/{attempt_id}/complete")
    def complete_study(attempt_id: str, request: CompleteStudy) -> StudyCompletion:
        try:
            result = service.complete(attempt_id, request.operation_id)
        except ValueError as error:
            raise HTTPException(409, str(error)) from error
        if not result:
            raise HTTPException(404, "Study attempt not found")
        return result

    @router.post("/cards/{card_id}/practise")
    def practise(card_id: str) -> StudyCard:
        card = service.free(card_id)
        if not card:
            raise HTTPException(404, "Card not found")
        return card

    @router.get("/cards/{card_id}/reference")
    def reference_audio(card_id: str) -> FileResponse:
        path = service.reference(card_id)
        if not path:
            raise HTTPException(404, "Card not found")
        return FileResponse(path, media_type="audio/wav")

    return router
