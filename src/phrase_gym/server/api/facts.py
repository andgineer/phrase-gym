from typing import Annotated

from fastapi import APIRouter, Query

from phrase_gym.server.api.paging import Limit, Offset
from phrase_gym.server.facts_contracts import PracticeFacts, StudiedPage
from phrase_gym.server.practice_facts import PracticeFactsService

LearningDate = Annotated[str, Query(pattern=r"^\d{4}-\d{2}-\d{2}$")]


def practice_facts_routes(facts: PracticeFactsService) -> APIRouter:
    router = APIRouter(prefix="/api/practice")

    @router.get("/facts")
    def practice_facts() -> PracticeFacts:
        return facts.facts()

    @router.get("/studied")
    def studied_phrases(
        learning_date: LearningDate,
        offset: Offset = 0,
        limit: Limit = 10,
    ) -> StudiedPage:
        return facts.studied(learning_date, offset, limit)

    return router
