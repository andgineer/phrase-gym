from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import ReturnPaceChoice, Settings
from phrase_gym.server.settings import SettingsService


def settings_routes(service: SettingsService) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.get("/settings")
    def settings() -> Settings:
        return service.settings()

    @router.get("/settings/return-paces")
    def return_paces() -> list[ReturnPaceChoice]:
        return service.return_paces()

    @router.put("/settings")
    def update_settings(settings: Settings) -> Settings:
        try:
            return service.update(settings)
        except ValueError as error:
            raise HTTPException(422, str(error)) from error

    return router
