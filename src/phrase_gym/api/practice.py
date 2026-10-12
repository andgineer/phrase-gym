from uuid import UUID

from fastapi import APIRouter, Request

from phrase_gym.api.recording_response import recording_response
from phrase_gym.contracts.practice import EventBatch, EventReceipt, RecordingInput, RecordingReceipt
from phrase_gym.contracts.sessions import PracticeBatchReceipt

router = APIRouter(prefix="/api/v1")


@router.post("/sessions/{session_id}/events", response_model=PracticeBatchReceipt)
def events(session_id: UUID, payload: EventBatch, request: Request):
    return request.app.state.practice.submit(session_id, payload)


@router.get("/sessions/{session_id}/events", response_model=list[EventReceipt])
def receipts(session_id: UUID, request: Request):
    return request.app.state.practice.receipts(session_id)


@router.post("/recordings", response_model=RecordingReceipt)
def recording(payload: RecordingInput, request: Request):
    return request.app.state.recordings.save(payload)


@router.get("/phrases/{target_id}/recordings", response_model=list[RecordingReceipt])
def recordings(target_id: UUID, request: Request):
    return request.app.state.recordings.list_for_target(target_id)


@router.get("/recordings/{recording_id}")
def play_recording(recording_id: UUID, request: Request):
    audio, mime_type, sha256 = request.app.state.recordings.delivery(recording_id)
    return recording_response(audio, mime_type, sha256, request.headers.get("range"))
