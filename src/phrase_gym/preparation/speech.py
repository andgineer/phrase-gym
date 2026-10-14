"""Whole-phrase speech, explicit language/accent voices and Serbian speech script."""

import asyncio
import base64
import hashlib
import json
import sys
import tempfile
from pathlib import Path
from typing import Literal
from uuid import UUID

import edge_tts
from pydantic import Field, model_validator
from sqlalchemy import select

from phrase_gym.contracts.base import Contract, Language
from phrase_gym.contracts.media import MediaMetadata, MediaUpload
from phrase_gym.errors import DomainError
from phrase_gym.media.service import MediaService
from phrase_gym.preparation import versions
from phrase_gym.serialization import canonical_json
from phrase_gym.tables import MediaRow


class SpeechVoice(Contract):
    engine: Literal["piper", "edge"]
    voice: str
    locale: str
    accent: str | None = None
    model_path: Path | None = None
    model_sha256: str | None = Field(default=None, pattern="^[a-f0-9]{64}$")
    config_sha256: str | None = Field(default=None, pattern="^[a-f0-9]{64}$")

    @model_validator(mode="after")
    def correct_voice(self) -> "SpeechVoice":
        if "serbski_institut" in self.voice:
            raise ValueError("Lower Sorbian is not a Serbian reference voice")
        if self.engine == "piper" and (
            not self.model_path or not self.model_sha256 or not self.config_sha256
        ):
            raise ValueError("Piper requires a model path and verified model/config SHA-256")
        if self.engine == "edge" and not self.voice.startswith(self.locale + "-"):
            raise ValueError("Edge voice must match its configured locale")
        return self


class SpeechSettings(Contract):
    voices: dict[Language, list[SpeechVoice]] = Field(default_factory=dict)
    timeout_seconds: float = Field(default=60, ge=1, le=120)

    @model_validator(mode="after")
    def same_language(self) -> "SpeechSettings":
        for language, voices in self.voices.items():
            if any(voice.locale.split("-")[0] != language for voice in voices):
                raise ValueError("Every fallback voice must speak the phrase language")
            if len({(v.locale, v.accent) for v in voices}) > 1:
                raise ValueError("Fallbacks must preserve the intended locale/accent")
        return self


def serbian_cyrillic(text: str) -> str:
    pairs = {"dž": "џ", "lj": "љ", "nj": "њ"}
    for latin, cyrillic in pairs.items():
        for source, target in (
            (latin, cyrillic),
            (latin.title(), cyrillic.upper()),
            (latin.upper(), cyrillic.upper()),
        ):
            text = text.replace(source, target)
    latin = "abcčćdđefghijklmnoprsštuvzžABCČĆDĐEFGHIJKLMNOPRSŠTUVZŽ"
    cyrillic = "абцчћдђефгхијклмнопрсштувзжАБЦЧЋДЂЕФГХИЈКЛМНОПРСШТУВЗЖ"
    return text.translate(str.maketrans(latin, cyrillic))


async def synthesize_piper(text: str, voice: SpeechVoice, timeout: float) -> bytes:
    assert voice.model_path is not None
    if hashlib.sha256(voice.model_path.read_bytes()).hexdigest() != voice.model_sha256:
        raise DomainError("Piper model does not match its configured checksum", 422)
    config_path = Path(str(voice.model_path) + ".json")
    config_bytes = config_path.read_bytes()
    if hashlib.sha256(config_bytes).hexdigest() != voice.config_sha256:
        raise DomainError("Piper configuration does not match its checksum", 422)
    locale = json.loads(config_bytes).get("language", {}).get("code", "").replace("_", "-")
    if locale != voice.locale:
        raise DomainError("Piper configuration locale does not match the requested voice", 422)
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "phrase.wav"
        process = await asyncio.create_subprocess_exec(
            sys.executable,
            "-m",
            "piper",
            "--model",
            str(voice.model_path),
            "--output-file",
            str(path),
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.DEVNULL,
            stderr=asyncio.subprocess.DEVNULL,
        )
        try:
            await asyncio.wait_for(process.communicate(text.encode()), timeout=timeout)
            if process.returncode != 0:
                raise DomainError("Piper failed to produce speech", 503)
            return path.read_bytes()
        finally:
            if process.returncode is None:
                process.kill()
                await process.wait()


class SpeechAdapter:
    def __init__(self, media: MediaService, settings: SpeechSettings):
        self.media = media
        self.settings = settings

    async def prepare(
        self,
        text: str,
        language: Language,
        source: str,
        role: Literal["reference", "partner"] = "reference",
    ) -> UUID:
        voices = self.settings.voices.get(language, [])
        for voice in voices:
            speech_text = serbian_cyrillic(text) if voice.locale == "sr-RS" else text
            metadata = MediaMetadata(
                language=language,
                text=text,
                role=role,
                origin="generated",
                source=source,
                engine=voice.engine,
                voice=voice.voice,
                locale=voice.locale,
                accent=voice.accent,
                speech_text=speech_text,
                preparation_version=versions.SPEECH_VERSION,
                settings={
                    "model_sha256": voice.model_sha256 or "",
                    "config_sha256": voice.config_sha256 or "",
                    "timeout_seconds": self.settings.timeout_seconds,
                },
            )
            with self.media.db.read() as transaction:
                cached = next(
                    (
                        row
                        for row in transaction.scalars(select(MediaRow))
                        if canonical_json(row.metadata_json)
                        == canonical_json(metadata.model_dump_json())
                        and self.media.available(row)
                    ),
                    None,
                )
                if cached:
                    return UUID(cached.id)
            try:
                if voice.engine == "piper":
                    content = await synthesize_piper(
                        speech_text,
                        voice,
                        self.settings.timeout_seconds,
                    )
                    mime_type = "audio/wav"
                else:
                    with tempfile.TemporaryDirectory() as directory:
                        path = Path(directory) / "phrase.mp3"
                        await asyncio.wait_for(
                            edge_tts.Communicate(speech_text, voice.voice).save(str(path)),
                            timeout=self.settings.timeout_seconds,
                        )
                        content = path.read_bytes()
                    mime_type = "audio/mpeg"
                return self.media.upload(
                    MediaUpload(
                        metadata=metadata,
                        mime_type=mime_type,
                        audio_base64=base64.b64encode(content).decode(),
                    ),
                ).id
            except (
                OSError,
                RuntimeError,
                TimeoutError,
                edge_tts.exceptions.EdgeTTSException,
                DomainError,
            ):
                continue
        raise DomainError("speech_unavailable: no matching configured voice completed", 503)
