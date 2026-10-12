from phrase_gym.contracts.base import Language
from phrase_gym.contracts.settings import Settings, SettingsUpdate
from phrase_gym.storage import AppSettings, Database


class SettingsService:
    def __init__(self, database: Database):
        self.db = database

    def settings(self) -> Settings:
        with self.db.read() as session:
            row = session.get(AppSettings, 1)
            return Settings.model_validate_json(row.payload) if row else Settings()

    def update_settings(self, settings: SettingsUpdate) -> Settings:
        with self.db.write() as session:
            session.merge(AppSettings(id=1, payload=settings.model_dump_json()))
        return Settings.model_validate(settings.model_dump())

    def remember_language(self, language: Language) -> Settings:
        with self.db.write() as session:
            row = session.get(AppSettings, 1)
            settings = Settings.model_validate_json(row.payload) if row else Settings()
            settings.last_learning_language = language
            session.merge(AppSettings(id=1, payload=settings.model_dump_json()))
        return settings
