from datetime import datetime

from phrase_gym.contracts.base import Contract


class DueSummary(Contract):
    due_count: int
    new_count: int
    preparing_count: int
    next_due_at: datetime | None
