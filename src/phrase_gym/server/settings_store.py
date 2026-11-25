import json
import sqlite3

from phrase_gym.server.contracts import Settings, TranslationChoices

# A stored document lacks settings added after it was saved and may keep retired ones.
DEFAULTS = {"return_pace": "balanced", "weekly_target": None, "weekly_target_paused": False}
# The learner's choices: English stays inside the language, German starts with native help.
TRANSLATION_DEFAULTS = {"en": False, "de": True}


def read_settings(connection: sqlite3.Connection) -> Settings:
    row = connection.execute("SELECT document FROM settings WHERE id=1").fetchone()
    stored = json.loads(row[0])
    known = {key: value for key, value in stored.items() if key in Settings.model_fields}
    translation = {
        language: choice
        for language, choice in (known.get("translation") or {}).items()
        if language in TranslationChoices.model_fields
    }
    known["translation"] = TRANSLATION_DEFAULTS | translation
    return Settings.model_validate(DEFAULTS | known)


def write_settings(connection: sqlite3.Connection, settings: Settings) -> None:
    connection.execute(
        "UPDATE settings SET document=? WHERE id=1",
        (settings.model_dump_json(),),
    )
