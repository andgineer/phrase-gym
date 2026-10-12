from fastapi import APIRouter, Request

from phrase_gym.contracts.settings import LanguagePreference, Settings, SettingsUpdate

router = APIRouter(prefix="/api/v1")


@router.get("/settings", response_model=Settings)
def get_settings(request: Request):
    return request.app.state.settings.settings()


@router.put("/settings", response_model=Settings)
def put_settings(settings: SettingsUpdate, request: Request):
    return request.app.state.settings.update_settings(settings)


@router.patch("/settings/language", response_model=Settings)
def remember_language(preference: LanguagePreference, request: Request):
    return request.app.state.settings.remember_language(preference.last_learning_language)
