from typing import Literal

from phrase_gym.contracts.base import Contract, Language


class Settings(Contract):
    native_language: Language | None = None
    ui_language_mode: Literal["native", "learning"] = "native"
    last_learning_language: Language | None = None


class LanguagePreference(Contract):
    last_learning_language: Language


class SettingsUpdate(Contract):
    native_language: Language
    ui_language_mode: Literal["native", "learning"] = "native"
    last_learning_language: Language | None = None
