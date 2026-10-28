from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import StudyHistoryItem
from phrase_gym.server.library import LibraryService


def library_routes(service: LibraryService) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.get("/library")
    def library() -> list[StudyHistoryItem]:
        return service.cards()

    @router.get("/study/history")
    def study_history() -> list[StudyHistoryItem]:
        return service.cards()

    @router.post("/cards/{card_id}/expose")
    def expose(card_id: str) -> dict[str, bool]:
        if not service.expose(card_id):
            raise HTTPException(404, "Card not found")
        return {"exposed": True}

    @router.post("/cards/{card_id}/suspend")
    def suspend(card_id: str) -> dict[str, bool]:
        if not service.suspend(card_id, True):
            raise HTTPException(404, "Card not found")
        return {"suspended": True}

    @router.post("/cards/{card_id}/restore")
    def restore(card_id: str) -> dict[str, bool]:
        if not service.suspend(card_id, False):
            raise HTTPException(404, "Card not found")
        return {"suspended": False}

    return router
