from typing import Annotated

from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import FileResponse

from phrase_gym.server.api.paging import Limit, Offset
from phrase_gym.server.contracts import (
    LessonPage,
    PhraseDetails,
    PhrasePage,
    RecordingPage,
)
from phrase_gym.server.lesson_choices import LessonChoices
from phrase_gym.server.library import LibraryService

SearchText = Annotated[str, Query(max_length=200)]


def library_routes(library: LibraryService, lessons: LessonChoices) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.get("/lessons")
    def lesson_choices(query: SearchText = "", offset: Offset = 0, limit: Limit = 20) -> LessonPage:
        return lessons.page(query, offset, limit)

    @router.get("/phrases")
    def find_phrases(
        query: SearchText = "",
        lesson_id: str | None = None,
        offset: Offset = 0,
        limit: Limit = 20,
    ) -> PhrasePage:
        return library.phrases(query, lesson_id, offset, limit)

    @router.get("/phrases/{phrase_id}")
    def phrase_details(phrase_id: str) -> PhraseDetails:
        found = library.phrase(phrase_id)
        if not found:
            raise HTTPException(404, "Phrase not found")
        return found

    @router.get("/phrases/{phrase_id}/recordings")
    def phrase_recordings(phrase_id: str, offset: Offset = 0, limit: Limit = 10) -> RecordingPage:
        found = library.recordings(phrase_id, offset, limit)
        if found is None:
            raise HTTPException(404, "Phrase not found")
        return found

    @router.get("/phrases/{phrase_id}/reference")
    def reference_audio(phrase_id: str) -> FileResponse:
        path = library.reference(phrase_id)
        if not path:
            raise HTTPException(404, "Phrase not found")
        return FileResponse(path, media_type="audio/wav")

    return router
