"""One bounded worker step, retaining concrete answers before validation/audio."""

import asyncio
import json
from http import HTTPStatus
from typing import Protocol
from uuid import UUID

from pydantic import ValidationError

from phrase_gym.contracts.preparation import ModelReview, PreparationDraft, SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.preparation.config import PreparationSettings
from phrase_gym.preparation.jobs import PreparationJobs
from phrase_gym.preparation.prompts import (
    PROMPT_VERSION,
    REVIEW_VERSION,
    generation_prompt,
    review_prompt,
)
from phrase_gym.preparation.speech import SpeechAdapter
from phrase_gym.preparation.validation import assemble


class TextProvider(Protocol):
    async def ask(self, prompt: str, operation: str) -> dict: ...


def parse_answer(text: str) -> dict:
    stripped = text.strip()
    if stripped.startswith("```"):
        stripped = stripped.split("\n", 1)[1].rsplit("```", 1)[0]
    return json.loads(stripped)


class PreparationWorker:
    def __init__(
        self,
        jobs: PreparationJobs,
        provider: TextProvider,
        speech: SpeechAdapter,
        settings_identity: dict | None = None,
    ):
        self.jobs = jobs
        self.provider = provider
        self.speech = speech
        self.settings_identity = settings_identity

    async def once(self, job_id: UUID | None = None) -> bool:
        claimed = self.jobs.claim(job_id)
        if claimed is None:
            return False
        job_id, token, payload, result = claimed
        try:
            if (
                payload["prompt_version"] != PROMPT_VERSION
                or payload["review_version"] != REVIEW_VERSION
            ):
                raise DomainError("Queued prompt version differs; enqueue a new revision", 422)
            if self.settings_identity is not None and (
                PreparationSettings.model_validate(payload["settings"]).identity()
                != PreparationSettings.model_validate(self.settings_identity).identity()
            ):
                raise DomainError("Worker configuration differs from the queued settings", 422)
            source = SourcePackage.model_validate(payload["source"])
            package = await self.prepare_content(source, result, job_id, token)
            await self.prepare_audio(source, package, result, job_id, token)
            result["package"] = package.model_dump(mode="json")
            self.jobs.checkpoint(job_id, token, result, complete=True)
        except asyncio.CancelledError:
            self.jobs.fail(job_id, token, "worker_interrupted: retry resumes retained checkpoints")
            raise
        except (DomainError, ValidationError, ValueError, KeyError, OSError) as error:
            code = (
                "provider_or_audio_failure"
                if isinstance(error, DomainError) and error.status == HTTPStatus.SERVICE_UNAVAILABLE
                else "draft_conformance_failure"
            )
            self.jobs.fail(job_id, token, code)
        return True

    async def prepare_content(self, source, result, job_id, token):
        if "generation" not in result:
            result["generation"] = await self.provider.ask(
                generation_prompt(source),
                "phrase-generation-" + source.language,
            )
            self.jobs.checkpoint(job_id, token, result)
        draft = PreparationDraft.model_validate(parse_answer(result["generation"]["text"]))
        package = assemble(source, draft)
        result["draft"] = draft.model_dump(mode="json")
        if "model_review" not in result:
            result["model_review"] = await self.provider.ask(
                review_prompt(source, draft),
                "phrase-review-" + source.language,
            )
            self.jobs.checkpoint(job_id, token, result)
        review = ModelReview.model_validate(parse_answer(result["model_review"]["text"]))
        expected = {(p.source_id, c.id) for p in draft.phrases for c in p.candidates}
        actual = {(c.source_id, c.candidate_id) for c in review.candidates}
        if actual != expected or len(review.candidates) != len(expected):
            raise DomainError("Model review must decide every candidate exactly once", 422)
        result["advisory_review"] = review.model_dump(mode="json")
        return package

    async def prepare_audio(self, source, package, result, job_id, token):
        audio = result.setdefault("audio", {})
        for target in package.targets:
            if target.reference_audio_id is None:
                key = "target:" + target.source_id
                if key not in audio:
                    audio[key] = str(
                        await self.speech.prepare(
                            target.text,
                            source.language,
                            source.namespace + ":" + target.source_id,
                        ),
                    )
                    self.jobs.checkpoint(job_id, token, result)
                target.reference_audio_id = UUID(audio[key])
        for dialogue in package.dialogues:
            for position, turn in enumerate(dialogue.turns):
                if turn.audio_id is None:
                    linked = next(
                        (t for t in package.targets if t.source_id == turn.target_source_id),
                        None,
                    )
                    if linked and linked.text == turn.text:
                        turn.audio_id = linked.reference_audio_id
                    else:
                        key = f"dialogue:{dialogue.source_id}:{position}"
                        if key not in audio:
                            audio[key] = str(
                                await self.speech.prepare(
                                    turn.text,
                                    source.language,
                                    source.namespace + ":" + key,
                                    "partner",
                                ),
                            )
                            self.jobs.checkpoint(job_id, token, result)
                        turn.audio_id = UUID(audio[key])
