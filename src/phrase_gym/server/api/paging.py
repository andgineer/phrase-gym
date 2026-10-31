from typing import Annotated

from fastapi import Query

from phrase_gym.server.contracts import PAGE_LIMIT

Offset = Annotated[int, Query(ge=0)]
Limit = Annotated[int, Query(ge=1, le=PAGE_LIMIT)]
