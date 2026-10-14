import base64
import json

import pytest

from phrase_gym.content.imports import ImportService
from phrase_gym.errors import DomainError
from phrase_gym.content.validation import PackageValidator
from phrase_gym.contracts.media import MediaUpload
from phrase_gym.contracts.preparation import SourcePackage, PreparationDraft
from phrase_gym.media.service import MediaService
from phrase_gym.preparation.jobs import PreparationJobs
from phrase_gym.storage import Database
from tests.integration.helpers import NOW, wav_bytes


@pytest.fixture
def prepared_runtime(tmp_path):
    db = Database(tmp_path / "data")
    db.migrate()
    clock = [NOW]
    jobs = PreparationJobs(db, lambda: clock[0])
    media = MediaService(db, tmp_path / "data")
    imports = ImportService(db, PackageValidator(media.available), lambda: clock[0])
    yield jobs, media, imports, clock
    db.close()


@pytest.fixture
def source():
    return SourcePackage.model_validate(
        {
            "namespace": "public-fixture",
            "language": "en",
            "native_language": "ru",
            "source_revision": "1",
            "phrases": [
                {
                    "source_id": "greeting",
                    "kind": "fixed_phrase",
                    "meaning_key": "greeting",
                    "recall_expression": "Not too bad, thanks. And yours?",
                    "text": "Not too bad, thanks. And yours?",
                    "purpose": "Reciprocate a greeting",
                }
            ],
            "lessons": [
                {"source_id": "lesson", "title": "Greetings", "target_source_ids": ["greeting"]}
            ],
        }
    )


@pytest.fixture
def draft():
    return PreparationDraft.model_validate(
        {
            "phrases": [
                {
                    "source_id": "greeting",
                    "chunks": [
                        {"id": "a", "text": "Not too bad, "},
                        {"id": "b", "text": "thanks. "},
                        {"id": "c", "text": "And yours?"},
                    ],
                    "required_chunks": ["a", "b", "c"],
                    "candidates": [
                        {
                            "id": "hide-a",
                            "prompt": "_____ thanks. And yours?",
                            "visible_chunks": ["b", "c"],
                            "competing_completions": ["Fine, thanks. And yours?"],
                            "distinguishing_anchors": "thanks. And yours?",
                            "recalled_material": "Not too bad",
                            "supplied_support": "Closing response",
                            "target_leak": False,
                        },
                        {
                            "id": "hide-c",
                            "prompt": "Not too bad, thanks. _____",
                            "visible_chunks": ["a", "b"],
                            "competing_completions": ["How about you?"],
                            "distinguishing_anchors": "Not too bad, thanks.",
                            "recalled_material": "And yours?",
                            "supplied_support": "Opening response",
                            "target_leak": False,
                        },
                    ],
                    "uncovered_chunks": {"b": "Too little identifying material"},
                    "explanation": {"meaning": "Неплохо, спасибо. А у тебя?"},
                    "concepts": ["greetings", "reciprocity"],
                }
            ]
        }
    )


class FakeProvider:
    def __init__(self, draft):
        self.draft = draft
        self.calls = []

    async def ask(self, prompt, operation):
        self.calls.append(operation)
        if "generation" in operation:
            value = self.draft.model_dump_json()
        else:
            value = json.dumps(
                {
                    "candidates": [
                        {
                            "source_id": p.source_id,
                            "candidate_id": c.id,
                            "acceptable": True,
                            "reason": "Software fixture only",
                        }
                        for p in self.draft.phrases
                        for c in p.candidates
                    ],
                    "help_errors": {},
                    "meaning_errors": {},
                }
            )
        return {
            "text": value,
            "model": "fake",
            "provider": "fake",
            "latency_seconds": 0,
            "usage": None,
        }


class FakeSpeech:
    def __init__(self, media):
        self.media = media
        self.calls = 0
        self.fail = False

    async def prepare(self, text, language, source, role="reference"):
        self.calls += 1
        if self.fail:
            raise DomainError("Unavailable", 503)
        return self.media.upload(
            MediaUpload.model_validate(
                {
                    "metadata": {
                        "language": language,
                        "text": text,
                        "role": role,
                        "origin": "test",
                        "source": source,
                        "locale": language + "-US",
                        "preparation_version": "fixture",
                    },
                    "mime_type": "audio/wav",
                    "audio_base64": base64.b64encode(wav_bytes()).decode(),
                }
            )
        ).id
