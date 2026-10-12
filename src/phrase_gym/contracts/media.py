from typing import Literal
from uuid import UUID

from pydantic import Field, model_validator

from phrase_gym.contracts.base import Contract, Language, Text


class MediaMetadata(Contract):
    language: Language
    text: Text
    role: Literal["reference", "cue", "partner"]
    origin: Literal["actor", "course", "generated", "test"]
    source: Text
    performer: str | None = None
    engine: str | None = None
    voice: str | None = None
    locale: str
    accent: str | None = None
    speech_text: str | None = None
    preparation_version: str
    settings: dict[str, str | int | float | bool] = Field(default_factory=dict)

    @model_validator(mode="after")
    def check_voice(self) -> "MediaMetadata":
        if self.locale.split("-")[0] != self.language:
            raise ValueError("Audio locale must match the learning language")
        if self.origin == "generated" and not all((self.engine, self.voice, self.speech_text)):
            raise ValueError("Generated audio requires engine, voice and exact speech input")
        if self.origin in ("actor", "course") and not self.performer:
            raise ValueError("Actor/course audio requires performer provenance")
        return self


class MediaUpload(Contract):
    metadata: MediaMetadata
    mime_type: Literal["audio/wav", "audio/mpeg", "audio/ogg", "audio/mp4"]
    audio_base64: str = Field(min_length=1, max_length=17000000)


class MediaAsset(Contract):
    id: UUID
    sha256: str
    byte_length: int
    mime_type: str
    metadata: MediaMetadata
    available: bool


class Reconciliation(Contract):
    missing_ids: list[UUID]
    orphan_hashes: list[str]
    removed_staging_files: int
