from fastapi import APIRouter, Request

from phrase_gym.contracts.settings import Settings, SettingsUpdate

router = APIRouter(prefix="/api/v1")


@router.get("/settings", response_model=Settings)
def get_settings(request: Request):
    return request.app.state.settings.settings()


@router.put("/settings", response_model=Settings)
def put_settings(settings: SettingsUpdate, request: Request):
    return request.app.state.settings.update_settings(settings)
