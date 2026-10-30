from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from phrase_gym.server.contracts import Settings
from phrase_gym.server.storage import Database


class SettingsService:
    def __init__(self, database: Database):
        self.database = database

    def settings(self) -> Settings:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT native_language,learning_timezone,card_language,help_language "
                "FROM settings WHERE id=1",
            ).fetchone()
            return Settings.model_validate(dict(row))

    def update(self, settings: Settings) -> Settings:
        try:
            ZoneInfo(settings.learning_timezone)
        except ZoneInfoNotFoundError as error:
            raise ValueError("Unknown learning timezone") from error
        with self.database.write() as connection:
            connection.execute(
                "UPDATE settings SET native_language=?, learning_timezone=?, "
                "card_language=?, help_language=? WHERE id=1",
                (
                    settings.native_language,
                    settings.learning_timezone,
                    settings.card_language,
                    settings.help_language,
                ),
            )
        return settings
