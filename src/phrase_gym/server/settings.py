from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from phrase_gym.server.contracts import ReturnPaceChoice, Settings
from phrase_gym.server.scheduling import RETENTION, good_gaps
from phrase_gym.server.settings_store import read_settings, write_settings
from phrase_gym.server.storage import Database

SHOWN_GAPS = 4


class SettingsService:
    def __init__(self, database: Database):
        self.database = database

    def settings(self) -> Settings:
        with self.database.read() as connection:
            return read_settings(connection)

    def update(self, settings: Settings) -> Settings:
        try:
            ZoneInfo(settings.learning_timezone)
        except ZoneInfoNotFoundError as error:
            raise ValueError("Unknown learning timezone") from error
        with self.database.write() as connection:
            write_settings(connection, settings)
        return settings

    def return_paces(self) -> list[ReturnPaceChoice]:
        return [
            ReturnPaceChoice(
                pace=pace,
                good_gaps_days=good_gaps(pace, SHOWN_GAPS),
                forget_one_in=round(1 / (1 - retention)),
            )
            for pace, retention in RETENTION.items()
        ]
