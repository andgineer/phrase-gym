from typing import Literal
from uuid import UUID

from phrase_gym.contracts.base import Contract, Language
from phrase_gym.contracts.content import DialogueInput, TargetInput


class Lesson(Contract):
    id: UUID
    language: Language
    title: str
    phrase_count: int


class Phrase(Contract):
    id: UUID
    revision: int
    language: Language
    content: TargetInput
    lessons: list[Lesson]
    audio_status: Literal["ready", "missing"]
    retrieval_ready: bool


class PhrasePage(Contract):
    items: list[Phrase]
    total: int
    offset: int
    limit: int


class LessonPage(Contract):
    items: list[Lesson]
    total: int
    offset: int
    limit: int


class Dialogue(Contract):
    id: UUID
    revision: int
    language: Language
    content: DialogueInput
    target_ids: dict[str, UUID]
    target_revisions: dict[str, int]


class DialoguePage(Contract):
    items: list[Dialogue]
    total: int
    offset: int
    limit: int
