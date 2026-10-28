from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app


@pytest.fixture
def client(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[TestClient]:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("PHRASE_GYM_FAKE_TTS", "1")
    monkeypatch.delenv("PHRASE_GYM_FAKE_TTS_FAILURES", raising=False)
    with TestClient(create_app()) as running:
        yield running


@pytest.fixture
def package() -> dict[str, object]:
    return {
        "format": "phrase-gym.package",
        "version": 1,
        "package_id": "course-meetings-v1",
        "source_id": "course-meetings",
        "source_label": "Course · Meetings",
        "source_link": "https://example.test/meetings",
        "learning_language": "en",
        "card_language": "en",
        "help_language": "en",
        "cards": [
            {
                "item_id": f"meeting-{index}",
                "answer": f"Complete phrase number {index}.",
                "explanation": "Use it to communicate naturally in a meeting.",
                "cues": [{"situation": f"Meeting situation number {index}."}],
                "accepted_variants": [],
            }
            for index in range(1, 6)
        ],
    }
