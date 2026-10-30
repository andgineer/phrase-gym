import hashlib
import json
import os
from pathlib import Path

from phrase_gym.server.contracts import RecordingReceipt, RecordingUpload
from phrase_gym.server.storage import Database, write_private


class RecordingConflictError(ValueError):
    pass


class DeletedPhraseError(ValueError):
    pass


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
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE id=?",
                (upload.attempt_id,),
            ).fetchone()
            if not attempt or attempt["phrase_id"] != upload.phrase_id:
                raise RecordingConflictError("Recording does not belong to this phrase and attempt")
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
                "INSERT INTO recordings (id,phrase_id,attempt_id,captured_at,mime_type,"
                "duration_ms,complete,capture_settings_json,path,byte_length,sha256,mode,"
                "cue_json,hint,answer_snapshot) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,NULL,?)",
                (
                    upload.id,
                    upload.phrase_id,
                    upload.attempt_id,
                    upload.captured_at,
                    upload.mime_type,
                    upload.duration_ms,
                    int(upload.complete),
                    json.dumps(upload.capture_settings),
                    str(path),
                    len(audio),
                    digest,
                    attempt["kind"],
                    attempt["cue_snapshot"],
                    attempt["answer_snapshot"],
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
