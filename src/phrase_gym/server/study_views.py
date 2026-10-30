import json
import sqlite3
from datetime import datetime

from phrase_gym.server.contracts import SavedRecording, StudyCard, StudyResponse
from phrase_gym.server.scheduling import effective_due


def saved_recording(row: sqlite3.Row) -> SavedRecording:
    return SavedRecording(
        id=row["id"],
        attempt_id=row["attempt_id"],
        captured_at=row["captured_at"],
        mime_type=row["mime_type"],
        duration_ms=row["duration_ms"],
        complete=bool(row["complete"]),
        byte_length=row["byte_length"],
        url=f"/api/recordings/{row['id']}/audio",
        mode=row["mode"],
        cue=json.loads(row["cue_json"]) if row["cue_json"] else None,
        hint=row["hint"],
        answer=row["answer_snapshot"],
    )


def recordings(connection: sqlite3.Connection, phrase_id: str) -> list[SavedRecording]:
    rows = connection.execute(
        "SELECT * FROM recordings WHERE phrase_id=? ORDER BY captured_at, id",
        (phrase_id,),
    ).fetchall()
    return [saved_recording(row) for row in rows]


def response(
    connection: sqlite3.Connection,
    card: sqlite3.Row | None,
    attempt: sqlite3.Row | None,
    instant: str,
) -> StudyResponse:
    total = connection.execute("SELECT COUNT(*) FROM phrases").fetchone()[0]
    scheduled = connection.execute(
        "SELECT * FROM phrases WHERE studied_at IS NOT NULL",
    ).fetchall()
    due_times = [effective_due(connection, row) for row in scheduled]
    now = datetime.fromisoformat(instant)
    due = sum(when <= now for when in due_times)
    later = [when for when in due_times if when > now]
    future = min(later).isoformat() if later else None
    message = None
    if not card:
        message = (
            f"Next return: {future[:10]}. Practise freely any time."
            if future
            else "Add your first phrase batch to begin."
            if not total
            else "Today's work is complete. Practise freely any time."
        )
    return StudyResponse(
        done=bool(total and not card),
        total=total,
        studied=len(scheduled),
        due_count=due,
        new_count=total - len(scheduled),
        next_return=future,
        message=message,
        card=study_card(connection, card, attempt) if card and attempt else None,
    )


def study_card(
    connection: sqlite3.Connection,
    row: sqlite3.Row,
    attempt: sqlite3.Row,
) -> StudyCard:
    revealed = attempt["kind"] != "review" or bool(attempt["revealed_at"])
    return StudyCard(
        id=row["id"],
        source_id=row["source_id"],
        item_id=row["item_id"],
        source_label=row["source_label"],
        source_link=row["source_link"],
        text=row["text"] if revealed else "",
        explanation=row["explanation"] if revealed else "",
        cues=json.loads(row["cues_json"]) if revealed else [],
        accepted_variants=json.loads(row["variants_json"]) if revealed else [],
        detailed_explanation=row["detailed_explanation"] if revealed else None,
        reference_url=f"/api/phrases/{row['id']}/reference" if revealed else "",
        attempt_id=attempt["id"],
        recordings=recordings(connection, row["id"]),
        kind=attempt["kind"],
        revealed=revealed,
        cue=json.loads(attempt["cue_snapshot"]),
    )
