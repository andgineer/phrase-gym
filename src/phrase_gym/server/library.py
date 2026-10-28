import json
from collections.abc import Callable
from datetime import UTC, datetime
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from phrase_gym.server.contracts import ReviewItem, Settings, StudyHistoryItem
from phrase_gym.server.exposure import expose_card
from phrase_gym.server.storage import Database
from phrase_gym.server.study_views import recordings


class LibraryService:
    def __init__(self, database: Database, clock: Callable[[], datetime] | None = None):
        self.database = database
        self.clock = clock or (lambda: datetime.now(UTC))

    def cards(self) -> list[StudyHistoryItem]:
        with self.database.read() as connection:
            cards = connection.execute(
                "SELECT * FROM cards ORDER BY studied_at IS NULL, studied_at DESC, created_at DESC",
            ).fetchall()
            result = []
            for card in cards:
                reviews = connection.execute(
                    "SELECT * FROM reviews WHERE card_id=? ORDER BY reviewed_at DESC",
                    (card["id"],),
                ).fetchall()
                result.append(
                    StudyHistoryItem(
                        id=card["id"],
                        item_id=card["item_id"],
                        answer=card["answer"],
                        source_label=card["source_label"],
                        reference_url=f"/api/cards/{card['id']}/reference",
                        recordings=recordings(connection, card["id"]),
                        suspended=bool(card["suspended"]),
                        due_at=card["due_at"],
                        reviews=[
                            ReviewItem(
                                reviewed_at=row["reviewed_at"],
                                cue=json.loads(row["cue_snapshot"]),
                                response_mode=row["response_mode"],
                                help_before=bool(row["help_before"]),
                                rating=row["rating"],
                                due_at=row["due_at"],
                            )
                            for row in reviews
                        ],
                    ),
                )
            return result

    def expose(self, card_id: str) -> bool:
        with self.database.write() as connection:
            card = connection.execute("SELECT id FROM cards WHERE id=?", (card_id,)).fetchone()
            if not card:
                return False
            expose_card(connection, card_id, self.clock())
            return True

    def suspend(self, card_id: str, value: bool) -> bool:
        with self.database.write() as connection:
            result = connection.execute(
                "UPDATE cards SET suspended=? WHERE id=?",
                (int(value), card_id),
            )
            if value:
                connection.execute(
                    "UPDATE attempts SET completed_at=? WHERE card_id=? AND completed_at IS NULL",
                    (datetime.now(UTC).isoformat(), card_id),
                )
            return bool(result.rowcount)

    def settings(self) -> Settings:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT native_language,learning_timezone,card_language,help_language "
                "FROM settings WHERE id=1",
            ).fetchone()
            return Settings.model_validate(dict(row))

    def update_settings(self, settings: Settings) -> Settings:
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
