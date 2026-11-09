from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from phrase_gym.server.contracts import ReturnPaceChoice, Settings
from phrase_gym.server.scheduling import RETENTION, good_return_days
from phrase_gym.server.settings_store import read_settings, write_settings
from phrase_gym.server.storage import Database

MONTH_DAYS = 30
YEAR_DAYS = 365


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
        choices = []
        for pace in RETENTION:
            days = good_return_days(pace, YEAR_DAYS)
            choices.append(
                ReturnPaceChoice(
                    pace=pace,
                    first_month_returns=sum(day <= MONTH_DAYS for day in days),
                    first_year_returns=len(days),
                ),
            )
        return choices
