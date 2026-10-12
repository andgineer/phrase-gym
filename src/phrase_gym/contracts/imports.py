from typing import Literal
from uuid import UUID

from phrase_gym.contracts.base import Contract


class ImportChange(Contract):
    source_id: str
    action: Literal["create", "update", "unchanged"]
    target_id: UUID | None = None
    base_revision: int | None = None


class ImportReport(Contract):
    id: UUID | None = None
    status: Literal["validated", "staged", "published", "rejected"]
    changes: list[ImportChange]
    errors: list[str]
    base_content_version: str


class PublishRequest(Contract):
    reviewed: Literal[True]
