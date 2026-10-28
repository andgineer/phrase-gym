from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import Settings
from phrase_gym.server.library import LibraryService


def settings_routes(service: LibraryService) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.get("/settings")
    def settings() -> Settings:
        return service.settings()

    @router.put("/settings")
    def update_settings(settings: Settings) -> Settings:
        try:
            return service.update_settings(settings)
        except ValueError as error:
            raise HTTPException(422, str(error)) from error

    return router
