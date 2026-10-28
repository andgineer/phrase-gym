import json
import sqlite3
from datetime import datetime

from phrase_gym.server.contracts import SavedRecording, StudyCard, StudyResponse
from phrase_gym.server.scheduling import effective_due


def recordings(connection: sqlite3.Connection, card_id: str) -> list[SavedRecording]:
    rows = connection.execute(
        "SELECT id, attempt_id, captured_at, mime_type, duration_ms, complete, byte_length "
        "FROM recordings WHERE card_id=? ORDER BY captured_at, id",
        (card_id,),
    ).fetchall()
    return [
        SavedRecording(
            id=row["id"],
            attempt_id=row["attempt_id"],
            captured_at=row["captured_at"],
            mime_type=row["mime_type"],
            duration_ms=row["duration_ms"],
            complete=bool(row["complete"]),
            byte_length=row["byte_length"],
            url=f"/api/recordings/{row['id']}/audio",
        )
        for row in rows
    ]


def response(
    connection: sqlite3.Connection,
    card: sqlite3.Row | None,
    attempt: sqlite3.Row | None,
    instant: str,
) -> StudyResponse:
    total = connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
    studied = connection.execute(
        "SELECT COUNT(*) FROM cards WHERE studied_at IS NOT NULL",
    ).fetchone()[0]
    scheduled = connection.execute(
        "SELECT * FROM cards WHERE suspended=0 AND studied_at IS NOT NULL",
    ).fetchall()
    due_times = [effective_due(connection, row) for row in scheduled]
    now = datetime.fromisoformat(instant)
    due = sum(when <= now for when in due_times)
    new = connection.execute(
        "SELECT COUNT(*) FROM cards WHERE suspended=0 AND studied_at IS NULL",
    ).fetchone()[0]
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
        studied=studied,
        due_count=due,
        new_count=new,
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
        answer=row["answer"] if revealed else "",
        explanation=row["explanation"] if revealed else "",
        cues=json.loads(row["cues_json"]) if revealed else [],
        accepted_variants=json.loads(row["variants_json"]) if revealed else [],
        detailed_explanation=row["detailed_explanation"] if revealed else None,
        reference_url=f"/api/cards/{row['id']}/reference" if revealed else "",
        attempt_id=attempt["id"],
        recordings=recordings(connection, row["id"]),
        kind=attempt["kind"],
        revealed=revealed,
        cue=json.loads(attempt["cue_snapshot"]),
    )
