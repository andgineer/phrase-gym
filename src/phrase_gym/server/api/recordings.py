import base64
import binascii
import hashlib
import json
import os

from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse

from phrase_gym.server.contracts import RecordingReceipt, RecordingUpload
from phrase_gym.server.storage import Database, write_private

MAX_RECORDING_BYTES = 12 * 1024 * 1024


def recording_routes(database: Database) -> APIRouter:
    router = APIRouter(prefix="/api/recordings")

    @router.post("")
    def save_recording(upload: RecordingUpload) -> RecordingReceipt:
        try:
            audio = base64.b64decode(upload.audio_base64, validate=True)
        except binascii.Error as error:
            raise HTTPException(422, "audio_base64 is invalid") from error
        if not audio:
            raise HTTPException(422, "Only nonempty takes can be saved")
        if len(audio) > MAX_RECORDING_BYTES:
            raise HTTPException(413, "Recording exceeds 12 MiB")
        digest = hashlib.sha256(audio).hexdigest()
        with database.write() as connection:
            existing = connection.execute(
                "SELECT * FROM recordings WHERE id=?",
                (upload.id,),
            ).fetchone()
            if existing:
                if existing["sha256"] != digest:
                    raise HTTPException(409, "Recording id was already used for different audio")
                return RecordingReceipt(
                    id=existing["id"],
                    byte_length=existing["byte_length"],
                    sha256=existing["sha256"],
                    complete=bool(existing["complete"]),
                )
            attempt = connection.execute(
                "SELECT card_id FROM attempts WHERE id=?",
                (upload.attempt_id,),
            ).fetchone()
            if not attempt or attempt["card_id"] != upload.card_id:
                raise HTTPException(409, "Recording does not belong to this card and attempt")
            directory = database.data_dir / "recordings" / upload.card_id
            directory.mkdir(parents=True, mode=0o700, exist_ok=True)
            directory.chmod(0o700)
            path = directory / upload.id
            temporary = path.with_suffix(".pending")
            temporary.unlink(missing_ok=True)
            write_private(temporary, audio)
            os.replace(temporary, path)
            connection.execute(
                "INSERT INTO recordings VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    upload.id,
                    upload.card_id,
                    upload.attempt_id,
                    upload.captured_at,
                    upload.mime_type,
                    upload.duration_ms,
                    int(upload.complete),
                    json.dumps(upload.capture_settings),
                    str(path),
                    len(audio),
                    digest,
                ),
            )
        return RecordingReceipt(
            id=upload.id,
            byte_length=len(audio),
            sha256=digest,
            complete=upload.complete,
        )

    @router.get("/{recording_id}/audio")
    def recording_audio(recording_id: str) -> FileResponse:
        with database.read() as connection:
            row = connection.execute(
                "SELECT path, mime_type FROM recordings WHERE id=?",
                (recording_id,),
            ).fetchone()
        if not row:
            raise HTTPException(404, "Recording not found")
        return FileResponse(row["path"], media_type=row["mime_type"])

    return router
