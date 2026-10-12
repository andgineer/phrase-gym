from typing import Literal

from phrase_gym.contracts.base import Contract, Language


class Settings(Contract):
    native_language: Language | None = None
    ui_language_mode: Literal["native", "learning"] = "native"


class SettingsUpdate(Contract):
    native_language: Language
    ui_language_mode: Literal["native", "learning"] = "native"
