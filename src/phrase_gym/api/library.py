from typing import Annotated
from uuid import UUID

from fastapi import APIRouter, Query, Request

from phrase_gym.api.parameters import Limit, Offset, Search
from phrase_gym.contracts.base import Language
from phrase_gym.contracts.library import Dialogue, DialoguePage, LessonPage, Phrase, PhrasePage

router = APIRouter(prefix="/api/v1")


@router.get("/lessons", response_model=LessonPage)
def lessons(
    language: Language,
    request: Request,
    q: Search = "",
    offset: Offset = 0,
    limit: Limit = 50,
):
    return request.app.state.library.lessons(language, q, offset, limit)


@router.get("/phrases/search", response_model=PhrasePage)
def phrases(
    language: Language,
    request: Request,
    q: Search = "",
    offset: Offset = 0,
    limit: Limit = 50,
):
    return request.app.state.library.phrases(language, q, offset, limit)


@router.get("/lessons/{lesson_id}/phrases", response_model=PhrasePage)
def lesson_phrases(
    lesson_id: UUID,
    language: Language,
    request: Request,
    offset: Offset = 0,
    limit: Limit = 50,
):
    return request.app.state.library.phrases(language, "", offset, limit, lesson_id)


@router.get("/phrases/{target_id}", response_model=Phrase)
def phrase(
    target_id: UUID,
    language: Language,
    request: Request,
    revision: Annotated[int | None, Query(ge=1)] = None,
):
    return request.app.state.library.phrase(target_id, language, revision)


@router.get("/dialogues", response_model=DialoguePage)
def dialogues(language: Language, request: Request, offset: Offset = 0, limit: Limit = 50):
    return request.app.state.library.dialogues(language, offset, limit)


@router.get("/dialogues/{dialogue_id}", response_model=Dialogue)
def dialogue(
    dialogue_id: UUID,
    language: Language,
    request: Request,
    revision: Annotated[int | None, Query(ge=1)] = None,
):
    return request.app.state.library.dialogue(dialogue_id, language, revision)
