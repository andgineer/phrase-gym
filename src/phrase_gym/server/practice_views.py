import json
import random
import sqlite3
from uuid import uuid4

from phrase_gym.server.import_contracts import Cue
from phrase_gym.server.practice_contracts import CardAnswer, PracticeCard, ViewAction
from phrase_gym.server.recording_views import view_recordings

REGULAR_ORIGINS = ("daily", "extra")


class PracticeConflictError(ValueError):
    pass


class StaleViewError(PracticeConflictError):
    pass


def load_view(connection: sqlite3.Connection) -> sqlite3.Row | None:
    return connection.execute("SELECT * FROM current_view WHERE id=1").fetchone()


def start_traversal(
    connection: sqlite3.Connection,
    day: str,
    origin: str,
    lesson_id: str | None,
) -> None:
    connection.execute(
        "INSERT OR REPLACE INTO current_view (id, traversal_id, learning_date, origin, "
        "lesson_id, skipped_json, phrase_id, view_token, kind, revealed, cue_index, "
        "hint_index, operations_json) VALUES (1,?,?,?,?,'[]',NULL,NULL,NULL,0,NULL,NULL,'{}')",
        (str(uuid4()), day, origin, lesson_id),
    )


def choose(rng: random.Random, count: int, avoid: int | None) -> int:
    options = [index for index in range(count) if index != avoid] or list(range(count))
    return rng.choice(options)


def show_phrase(
    connection: sqlite3.Connection,
    phrase: sqlite3.Row,
    kind: str,
    rng: random.Random,
) -> None:
    cue_index = None
    if kind == "recall":
        cue_index = choose(rng, len(json.loads(phrase["cues_json"])), phrase["last_cue"])
        connection.execute("UPDATE phrases SET last_cue=? WHERE id=?", (cue_index, phrase["id"]))
    # Every appearance gets a fresh token, even when the same phrase is shown again.
    connection.execute(
        "UPDATE current_view SET phrase_id=?, view_token=?, kind=?, revealed=?, cue_index=?, "
        "hint_index=NULL, operations_json='{}' WHERE id=1",
        (phrase["id"], str(uuid4()), kind, int(kind == "first_study"), cue_index),
    )


def clear_phrase(connection: sqlite3.Connection) -> None:
    connection.execute(
        "UPDATE current_view SET phrase_id=NULL, view_token=NULL, kind=NULL, revealed=0, "
        "cue_index=NULL, hint_index=NULL, operations_json='{}' WHERE id=1",
    )


def require_view(connection: sqlite3.Connection, day: str, action: ViewAction) -> sqlite3.Row:
    view = load_view(connection)
    if (
        view is None
        or view["learning_date"] != day
        or view["view_token"] != action.view_token
        or view["phrase_id"] != action.phrase_id
    ):
        raise StaleViewError("This card is no longer current. Open Daily to continue.")
    return view


def view_phrase(connection: sqlite3.Connection, view: sqlite3.Row) -> sqlite3.Row:
    return connection.execute(
        "SELECT phrases.*, lessons.title AS lesson_title FROM phrases "
        "JOIN lessons ON lessons.id=phrases.lesson_id WHERE phrases.id=?",
        (view["phrase_id"],),
    ).fetchone()


def practice_card(
    connection: sqlite3.Connection,
    view: sqlite3.Row,
    phrase: sqlite3.Row,
) -> PracticeCard:
    cues = json.loads(phrase["cues_json"])
    hints = json.loads(phrase["hints_json"])
    revealed = bool(view["revealed"])
    answer = None
    if revealed:
        answer = CardAnswer(
            text=phrase["text"],
            explanation=phrase["explanation"],
            detailed_explanation=phrase["detailed_explanation"],
            accepted_variants=json.loads(phrase["variants_json"]),
            usage_contexts=json.loads(phrase["contexts_json"]),
            reference_url=f"/api/phrases/{phrase['id']}/reference",
        )
    return PracticeCard(
        view_token=view["view_token"],
        phrase_id=phrase["id"],
        mode=view["origin"],
        kind=view["kind"],
        revealed=revealed,
        cue=Cue(**cues[view["cue_index"]]) if view["cue_index"] is not None else None,
        other_cue_available=view["kind"] == "recall" and len(cues) > 1,
        hint=hints[view["hint_index"]] if view["hint_index"] is not None else None,
        answer=answer,
        # Saved takes carry an answer snapshot, so the hidden front withholds them.
        recordings=view_recordings(connection, view["view_token"]) if revealed else [],
    )
