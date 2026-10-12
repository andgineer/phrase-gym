from uuid import UUID

from fastapi import APIRouter, Request

from phrase_gym.contracts.base import Language
from phrase_gym.contracts.readiness import DueSummary
from phrase_gym.contracts.sessions import PracticeSession, SessionStart, SessionTransition

router = APIRouter(prefix="/api/v1/sessions")


@router.post("", response_model=PracticeSession)
def start(payload: SessionStart, request: Request):
    return request.app.state.sessions.start(payload)


@router.get("/current", response_model=PracticeSession | None)
def current(request: Request):
    return request.app.state.sessions.current()


@router.get("/readiness", response_model=DueSummary)
def readiness(language: Language, request: Request, lesson_id: UUID | None = None):
    return request.app.state.readiness.summary(language, lesson_id)


@router.get("/{session_id}", response_model=PracticeSession)
def session(session_id: UUID, request: Request):
    return request.app.state.sessions.get(session_id)


@router.patch("/{session_id}", response_model=PracticeSession)
def transition(session_id: UUID, payload: SessionTransition, request: Request):
    return request.app.state.sessions.transition(session_id, payload)
