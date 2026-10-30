import base64
import binascii

from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse

from phrase_gym.server.contracts import RecordingReceipt, RecordingUpload
from phrase_gym.server.recording_store import (
    DeletedPhraseError,
    RecordingConflictError,
    RecordingStore,
)

MAX_RECORDING_BYTES = 12 * 1024 * 1024


def recording_routes(store: RecordingStore) -> APIRouter:
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
        try:
            return store.save(upload, audio)
        except DeletedPhraseError as error:
            raise HTTPException(410, str(error)) from error
        except RecordingConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.get("/{recording_id}/audio")
    def recording_audio(recording_id: str) -> FileResponse:
        found = store.audio(recording_id)
        if not found:
            raise HTTPException(404, "Recording not found")
        path, mime_type = found
        return FileResponse(path, media_type=mime_type)

    return router
