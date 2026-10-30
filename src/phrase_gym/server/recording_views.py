import json
import sqlite3
from datetime import datetime

from phrase_gym.server.contracts import SavedRecording
from phrase_gym.server.scheduling import learning_day


def saved_recording(connection: sqlite3.Connection, row: sqlite3.Row) -> SavedRecording:
    return SavedRecording(
        id=row["id"],
        captured_at=row["captured_at"],
        learning_date=learning_day(connection, datetime.fromisoformat(row["captured_at"])),
        mime_type=row["mime_type"],
        duration_ms=row["duration_ms"],
        complete=bool(row["complete"]),
        byte_length=row["byte_length"],
        url=f"/api/recordings/{row['id']}/audio",
        mode=row["mode"],
        kind=row["kind"],
        cue=json.loads(row["cue_json"]) if row["cue_json"] else None,
        hint=row["hint"],
        answer=row["answer_snapshot"],
    )


def view_recordings(connection: sqlite3.Connection, view_token: str) -> list[SavedRecording]:
    rows = connection.execute(
        "SELECT * FROM recordings WHERE view_token=? ORDER BY captured_at, id",
        (view_token,),
    ).fetchall()
    return [saved_recording(connection, row) for row in rows]
