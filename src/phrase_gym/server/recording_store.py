import hashlib
import json
import os
import sqlite3
from pathlib import Path

from phrase_gym.server.contracts import RecordingReceipt, RecordingUpload
from phrase_gym.server.storage import Database, write_private


class RecordingConflictError(ValueError):
    pass


class DeletedPhraseError(ValueError):
    pass


def _require_shown_support(upload: RecordingUpload, phrase: sqlite3.Row) -> None:
    cues = json.loads(phrase["cues_json"])
    if upload.cue and upload.cue.model_dump(exclude_none=True) not in cues:
        raise RecordingConflictError("Recording cue is not one of this phrase's cues")
    if upload.hint is not None and upload.hint not in json.loads(phrase["hints_json"]):
        raise RecordingConflictError("Recording hint is not one of this phrase's hints")


class RecordingStore:
    def __init__(self, database: Database):
        self.database = database

    def save(self, upload: RecordingUpload, audio: bytes) -> RecordingReceipt:
        digest = hashlib.sha256(audio).hexdigest()
        with self.database.write() as connection:
            existing = connection.execute(
                "SELECT * FROM recordings WHERE id=?",
                (upload.id,),
            ).fetchone()
            if existing:
                if existing["sha256"] != digest:
                    raise RecordingConflictError(
                        "Recording id was already used for different audio",
                    )
                return RecordingReceipt(
                    id=existing["id"],
                    byte_length=existing["byte_length"],
                    sha256=existing["sha256"],
                    complete=bool(existing["complete"]),
                )
            if connection.execute(
                "SELECT 1 FROM deleted_phrases WHERE id=?",
                (upload.phrase_id,),
            ).fetchone():
                raise DeletedPhraseError(
                    "This phrase was deleted, so the recording was not saved. "
                    "Discard it from this device.",
                )
            phrase = connection.execute(
                "SELECT text, cues_json, hints_json FROM phrases WHERE id=?",
                (upload.phrase_id,),
            ).fetchone()
            if not phrase:
                raise RecordingConflictError("Recording does not belong to a known phrase")
            _require_shown_support(upload, phrase)
            directory = self.database.data_dir / "recordings" / upload.phrase_id
            directory.mkdir(parents=True, mode=0o700, exist_ok=True)
            directory.chmod(0o700)
            path = directory / upload.id
            if path.resolve().parent != directory.resolve():
                raise RecordingConflictError("Recording id must name a file in its phrase folder")
            temporary = directory / f"{upload.id}.pending"
            temporary.unlink(missing_ok=True)
            write_private(temporary, audio)
            os.replace(temporary, path)
            connection.execute(
                "INSERT INTO recordings (id,phrase_id,view_token,captured_at,mime_type,"
                "duration_ms,complete,capture_settings_json,path,byte_length,sha256,mode,"
                "kind,cue_json,hint,answer_snapshot) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (
                    upload.id,
                    upload.phrase_id,
                    upload.view_token,
                    upload.captured_at,
                    upload.mime_type,
                    upload.duration_ms,
                    int(upload.complete),
                    json.dumps(upload.capture_settings),
                    str(path),
                    len(audio),
                    digest,
                    upload.mode,
                    upload.kind,
                    upload.cue.model_dump_json(exclude_none=True) if upload.cue else None,
                    upload.hint,
                    phrase["text"],
                ),
            )
        return RecordingReceipt(
            id=upload.id,
            byte_length=len(audio),
            sha256=digest,
            complete=upload.complete,
        )

    def audio(self, recording_id: str) -> tuple[Path, str] | None:
        with self.database.read() as connection:
            row = connection.execute(
                "SELECT path, mime_type FROM recordings WHERE id=?",
                (recording_id,),
            ).fetchone()
        return (Path(row["path"]), row["mime_type"]) if row else None
