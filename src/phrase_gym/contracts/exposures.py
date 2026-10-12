from typing import Literal
from uuid import UUID

from pydantic import Field

from phrase_gym.contracts.base import Contract, Language


class ExposureInput(Contract):
    id: UUID
    target_id: UUID
    revision: int = Field(ge=1)
    language: Language
    kind: Literal["answer", "explanation", "reference"]


class ExposureReceipt(Contract):
    id: UUID
    received_at: str
