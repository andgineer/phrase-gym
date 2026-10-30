from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from phrase_gym.server.contracts import Settings
from phrase_gym.server.storage import Database

COLUMNS = (
    "native_language",
    "learning_timezone",
    "card_language",
    "help_language",
    "daily_due_limit",
    "daily_new_limit",
)


class SettingsService:
    def __init__(self, database: Database):
        self.database = database

    def settings(self) -> Settings:
        with self.database.read() as connection:
            row = connection.execute(
                f"SELECT {','.join(COLUMNS)} FROM settings WHERE id=1",  # noqa: S608
            ).fetchone()
            return Settings.model_validate(dict(row))

    def update(self, settings: Settings) -> Settings:
        try:
            ZoneInfo(settings.learning_timezone)
        except ZoneInfoNotFoundError as error:
            raise ValueError("Unknown learning timezone") from error
        with self.database.write() as connection:
            assignments = ", ".join(f"{column}=?" for column in COLUMNS)
            connection.execute(
                f"UPDATE settings SET {assignments} WHERE id=1",  # noqa: S608
                tuple(getattr(settings, column) for column in COLUMNS),
            )
        return settings
