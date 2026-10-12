from phrase_gym.contracts.settings import Settings, SettingsUpdate
from phrase_gym.storage import Database
from phrase_gym.tables import AppSettings


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
